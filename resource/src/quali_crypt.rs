use std::io::Read;
use std::sync::LazyLock;

use aes::Aes128;
use aes::cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7};
use cbc::{Decryptor, Encryptor};
use flate2::read::ZlibDecoder;
use md5::{Digest, Md5};

// envelope: [hdr_size:2 LE][compress:1][keylen:1][nonce:keylen][AES-CBC ciphertext]
// AES key = MD5("BJYZ0w3DJm")
// IV = MD5(keyword || nonce)
// compress == 1 means zlib

const SECRET_KEYWORD: &[u8] = b"BJYZ0w3DJm";
const MAX_KEY_LEN: usize = 128;

static AES_KEY: LazyLock<[u8; 16]> = LazyLock::new(|| Md5::digest(SECRET_KEYWORD).into());

fn iv_for(nonce: &[u8]) -> [u8; 16] {
    let mut buf = Vec::with_capacity(SECRET_KEYWORD.len() + nonce.len());
    buf.extend_from_slice(SECRET_KEYWORD);
    buf.extend_from_slice(nonce);
    Md5::digest(&buf).into()
}

pub fn decrypt_body(body: &[u8]) -> Result<Vec<u8>, String> {
    if body.len() < 6 {
        return Err(format!("body too small ({} bytes)", body.len()));
    }

    let hdr_size = u16::from_le_bytes([body[0], body[1]]) as usize;
    let compress = body[2];
    let keylen = body[3] as usize;

    if !(1..=MAX_KEY_LEN).contains(&keylen) {
        return Err(format!("invalid key length {keylen}"));
    }

    if hdr_size != keylen + 2 {
        return Err(format!(
            "header size {hdr_size} != keylen+2 ({})",
            keylen + 2
        ));
    }

    let total_hdr = hdr_size + 2;
    if body.len() < total_hdr {
        return Err(format!(
            "header ({total_hdr}) larger than body ({})",
            body.len()
        ));
    }

    let nonce = &body[4..4 + keylen];
    let ciphertext = &body[total_hdr..];
    let plain = Decryptor::<Aes128>::new(&(*AES_KEY).into(), &iv_for(nonce).into())
        .decrypt_padded_vec::<Pkcs7>(ciphertext)
        .map_err(|e| format!("AES decrypt failed: {e}"))?;

    if compress == 1 {
        let mut dec = ZlibDecoder::new(&plain[..]);
        let mut out = Vec::new();
        dec.read_to_end(&mut out)
            .map_err(|e| format!("zlib inflate failed: {e}"))?;
        Ok(out)
    } else {
        Ok(plain)
    }
}

pub fn encrypt_body(plain: &[u8]) -> Vec<u8> {
    let nonce: [u8; 16] = rand::random();
    let ciphertext = Encryptor::<Aes128>::new(&(*AES_KEY).into(), &iv_for(&nonce).into())
        .encrypt_padded_vec::<Pkcs7>(plain);

    let keylen = nonce.len() as u8;
    let hdr_size = keylen + 2;

    let mut out = Vec::with_capacity(4 + nonce.len() + ciphertext.len());
    out.extend_from_slice(&(hdr_size as u16).to_le_bytes());
    out.push(0); // compress = 0
    out.push(keylen);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    out
}

// transform every gRPC frame payload; frames are [flag:1][length:4 BE][payload].
pub fn map_frames(
    data: &[u8],
    f: impl Fn(&[u8]) -> Result<Vec<u8>, String>,
) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(data.len() + 16);
    let mut pos = 0usize;

    while pos < data.len() {
        if data.len() - pos < 5 {
            return Err("truncated gRPC frame header".into());
        }

        let flag = data[pos];
        let len = u32::from_be_bytes(data[pos + 1..pos + 5].try_into().unwrap()) as usize;

        pos += 5;
        if data.len() - pos < len {
            return Err(format!(
                "truncated gRPC frame ({} < {len})",
                data.len() - pos
            ));
        }

        let payload = &data[pos..pos + len];
        pos += len;

        // zero-length frames carry Empty messages and pass through unchanged.
        if len == 0 {
            out.push(flag);
            out.extend_from_slice(&[0u8; 4]);
            continue;
        }

        let mapped = f(payload)?;
        out.push(flag);
        out.extend_from_slice(&(mapped.len() as u32).to_be_bytes());
        out.extend_from_slice(&mapped);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let plain = b"hello world this is a protobuf-ish payload with some length";
        let body = encrypt_body(plain);
        assert_eq!(decrypt_body(&body).unwrap(), plain);
    }

    #[test]
    fn map_frames_round_trip() {
        let frames = {
            let mut v = Vec::new();
            for msg in [&b"first-message"[..], &b"second"[..]] {
                v.push(0u8);
                v.extend_from_slice(&(msg.len() as u32).to_be_bytes());
                v.extend_from_slice(msg);
            }
            v
        };
        let enc = map_frames(&frames, |p| Ok(encrypt_body(p))).unwrap();
        let dec = map_frames(&enc, decrypt_body).unwrap();
        assert_eq!(dec, frames);
    }
}
