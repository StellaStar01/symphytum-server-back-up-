use aes::{
    Aes128,
    cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7},
};
use bytes::Bytes;
use cbc::{Decryptor, Encryptor};
use flate2::read::ZlibDecoder;
use http::{Request, Response};
use http_body_util::BodyExt;
use md5::{Digest, Md5};
use rand::RngExt;
use std::io::Read;
use std::sync::LazyLock;
use tonic::Status;
use tonic::body::Body;
use tonic_middleware::{Middleware, ServiceBound};

type Aes128CbcEnc = Encryptor<Aes128>;
type Aes128CbcDec = Decryptor<Aes128>;

const SECRET_KEYWORD: &[u8] = b"BJYZ0w3DJm";

static AES_KEY: LazyLock<[u8; 16]> = LazyLock::new(|| {
    let mut hasher = Md5::new();
    hasher.update(SECRET_KEYWORD);
    hasher.finalize().into()
});

fn derive_iv(key: &[u8]) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(SECRET_KEYWORD);
    hasher.update(key);
    hasher.finalize().into()
}

fn aes_cbc_decrypt(
    key: &[u8; 16],
    iv: &[u8; 16],
    ciphertext: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let decryptor = Aes128CbcDec::new(key.into(), iv.into());
    let mut buf = ciphertext.to_vec();
    let pt = decryptor
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|_| "Decryption or padding check failed")?;
    Ok(pt.to_vec())
}

fn aes_cbc_encrypt(
    key: &[u8; 16],
    iv: &[u8; 16],
    plaintext: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let encryptor = Aes128CbcEnc::new(key.into(), iv.into());
    let pos = plaintext.len();
    let padding_len = 16 - (pos % 16);
    let total_len = pos + padding_len;
    let mut buf = vec![0u8; total_len];
    buf[..pos].copy_from_slice(plaintext);
    let ct_len = encryptor
        .encrypt_padded::<Pkcs7>(&mut buf, pos)
        .map_err(|_| "Encryption padding failed")?
        .len();
    buf.truncate(ct_len);
    Ok(buf)
}

fn decrypt_payload(msg: &[u8]) -> Result<Vec<u8>, &'static str> {
    if msg.len() < 6 {
        return Err("Message too short");
    }
    let hdr_size = (msg[0] as usize) | ((msg[1] as usize) << 8);
    let hdr_compress = msg[2];
    let keylen = msg[3] as usize;
    if keylen < 1 || keylen > 128 || hdr_size != keylen + 2 {
        return Err("Invalid header size or key length");
    }
    let total_hdr = hdr_size + 2;
    if msg.len() < total_hdr {
        return Err("Message shorter than header");
    }
    let key = &msg[4..4 + keylen];
    let ct = &msg[total_hdr..];

    let iv = derive_iv(key);
    let mut pt = aes_cbc_decrypt(&AES_KEY, &iv, ct)?;

    if hdr_compress != 0 {
        let mut decoder = ZlibDecoder::new(&pt[..]);
        let mut decompressed = Vec::with_capacity(pt.len() * 2);
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|_| "Zlib decompression failed")?;
        pt = decompressed;
    }
    Ok(pt)
}

fn decrypt_grpc_body(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut decrypted_data = Vec::with_capacity(data.len());
    let mut pos = 0;
    while pos < data.len() {
        let remaining = &data[pos..];
        if remaining.len() < 5 {
            break;
        }
        let frame_len = ((remaining[1] as usize) << 24)
            | ((remaining[2] as usize) << 16)
            | ((remaining[3] as usize) << 8)
            | (remaining[4] as usize);
        if remaining.len() < 5 + frame_len {
            break;
        }
        let msg = &remaining[5..5 + frame_len];
        pos += 5 + frame_len;

        let decrypted_msg = decrypt_payload(msg)?;
        decrypted_data.push(0);
        let len_bytes = (decrypted_msg.len() as u32).to_be_bytes();
        decrypted_data.extend_from_slice(&len_bytes);
        decrypted_data.extend_from_slice(&decrypted_msg);
    }
    Ok(decrypted_data)
}

fn encrypt_payload(pt: Vec<u8>) -> Result<Vec<u8>, &'static str> {
    let hdr_compress = 0;
    let mut key = [0u8; 16];
    rand::rng().fill(&mut key[..]);

    let iv = derive_iv(&key);
    let ct = aes_cbc_encrypt(&AES_KEY, &iv, &pt)?;

    let keylen = key.len();
    let hdr_size = keylen + 2;
    let total_len = 2 + 1 + 1 + keylen + ct.len();
    let mut msg = vec![0u8; total_len];
    msg[0..2].copy_from_slice(&(hdr_size as u16).to_le_bytes());
    msg[2] = hdr_compress;
    msg[3] = keylen as u8;
    msg[4..20].copy_from_slice(&key);
    msg[20..].copy_from_slice(&ct);

    Ok(msg)
}

fn encrypt_grpc_body(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut encrypted_data = Vec::with_capacity(data.len() + 64);
    let mut pos = 0;
    while pos < data.len() {
        let remaining = &data[pos..];
        if remaining.len() < 5 {
            break;
        }
        let frame_len = ((remaining[1] as usize) << 24)
            | ((remaining[2] as usize) << 16)
            | ((remaining[3] as usize) << 8)
            | (remaining[4] as usize);
        if remaining.len() < 5 + frame_len {
            break;
        }
        let msg = &remaining[5..5 + frame_len];
        pos += 5 + frame_len;

        let encrypted_msg = encrypt_payload(msg.to_vec())?;
        encrypted_data.push(0);
        let len_bytes = (encrypted_msg.len() as u32).to_be_bytes();
        encrypted_data.extend_from_slice(&len_bytes);
        encrypted_data.extend_from_slice(&encrypted_msg);
    }
    Ok(encrypted_data)
}

#[derive(Default, Clone)]
pub struct QualiCrypt;

#[tonic::async_trait]
impl<S> Middleware<S> for QualiCrypt
where
    S: ServiceBound,
    S::Future: Send,
{
    async fn call(&self, req: Request<Body>, mut service: S) -> Result<Response<Body>, S::Error> {
        let (parts, body) = req.into_parts();

        let collected: http_body_util::Collected<Bytes> = match body.collect().await {
            Ok(c) => c,
            Err(e) => {
                let status =
                    Status::invalid_argument(format!("Failed to read request body: {}", e));
                return Ok(status.into_http());
            }
        };
        let encrypted_bytes = collected.to_bytes();

        let decrypted_data = match decrypt_grpc_body(&encrypted_bytes) {
            Ok(d) => d,
            Err(err) => {
                let status = Status::invalid_argument(format!("Decryption error: {}", err));
                return Ok(status.into_http());
            }
        };

        let decrypted_body = Body::new(http_body_util::Full::new(Bytes::from(decrypted_data)));
        let decrypted_req = Request::from_parts(parts, decrypted_body);

        let res = service.call(decrypted_req).await?;
        let (res_parts, res_body) = res.into_parts();

        let res_collected: http_body_util::Collected<Bytes> = match res_body.collect().await {
            Ok(c) => c,
            Err(e) => {
                let status = Status::internal(format!("Failed to read response body: {}", e));
                return Ok(status.into_http());
            }
        };
        let decrypted_res_bytes = res_collected.to_bytes();

        let encrypted_res_data = match encrypt_grpc_body(&decrypted_res_bytes) {
            Ok(d) => d,
            Err(err) => {
                let status = Status::internal(format!("Encryption error: {}", err));
                return Ok(status.into_http());
            }
        };

        let boxed_body = Body::new(http_body_util::Full::new(Bytes::from(encrypted_res_data)));

        Ok(Response::from_parts(res_parts, boxed_body))
    }
}
