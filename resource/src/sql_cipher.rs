use aes::Aes256;
use aes::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::NoPadding};
use cbc::Decryptor;

const PAGE_SIZE: usize = 4096;
const IV_SIZE: usize = 16;
const HMAC_SIZE: usize = 64;
const RESERVE: usize = IV_SIZE + HMAC_SIZE;
const SALT_SIZE: usize = 16;

/// decrypt a sqlcipher db file into a plain sqlite db
pub fn decrypt_file(file: &[u8], hex_key: &str) -> Result<Vec<u8>, String> {
    let key: [u8; 32] = hex_decode(hex_key)?
        .try_into()
        .map_err(|_| "crypto_key must be 64 hex chars".to_string())?;

    if file.len() < PAGE_SIZE || file.len() % PAGE_SIZE != 0 {
        return Err(format!(
            "unexpected file size {} (not a multiple of {PAGE_SIZE})",
            file.len()
        ));
    }

    let mut out = Vec::with_capacity(file.len());
    for (idx, page) in file.chunks_exact(PAGE_SIZE).enumerate() {
        let offset = if idx == 0 { SALT_SIZE } else { 0 };
        let usable = PAGE_SIZE - offset - RESERVE;
        let ciphertext = &page[offset..offset + usable];
        let iv = &page[offset + usable..offset + usable + IV_SIZE];
        let _ = iv;

        let iv_arr: [u8; IV_SIZE] = iv.try_into().map_err(|_| "bad iv length")?;
        let plain = Decryptor::<Aes256>::new(&key.into(), &iv_arr.into())
            .decrypt_padded_vec::<NoPadding>(ciphertext)
            .map_err(|e| format!("page {} decrypt failed: {e}", idx + 1))?;

        if idx == 0 {
            // replace the salt region with the standard sqlite header magic
            out.extend_from_slice(b"SQLite format 3\x00");
            out.extend_from_slice(&plain);
        } else {
            out.extend_from_slice(&plain);
        }
        // reserve bytes (iv + hmac) stay as-is
        out.extend_from_slice(&page[offset + usable..PAGE_SIZE]);
    }
    Ok(out)
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("odd hex length".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("bad hex: {e}")))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_decode_roundtrip() {
        assert_eq!(hex_decode("00ff10").unwrap(), vec![0x00, 0xff, 0x10]);
        assert!(hex_decode("0").is_err());
    }
}
