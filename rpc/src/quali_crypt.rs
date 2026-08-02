use std::io::Read;
use std::pin::Pin;
use std::sync::LazyLock;
use std::task::{Context, Poll};

use aes::Aes128;
use aes::cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7};
use cbc::{Decryptor, Encryptor};
use md5::{Digest, Md5};

use flate2::read::ZlibDecoder;
use tonic::body::Body;
use tonic::codegen::{Bytes, http};

// envelope:
// [hdr_size:2 LE][compress:1][keylen:1][nonce:keylen][AES-CBC ciphertext]
//
// with `hdr_size == keylen + 2`, AES key = `MD5("BJYZ0w3DJm")`, and
// `IV = MD5("BJYZ0w3DJm" || nonce)`. `compress == 1` means zlib

const SECRET_KEYWORD: &[u8] = b"BJYZ0w3DJm";
const MAX_KEY_LEN: usize = 128;

// AES key = MD5(keyword).
static AES_KEY: LazyLock<[u8; 16]> = LazyLock::new(|| Md5::digest(SECRET_KEYWORD).into());

// IV = MD5(keyword || nonce).
fn iv_for(nonce: &[u8]) -> [u8; 16] {
    let mut buf = Vec::with_capacity(SECRET_KEYWORD.len() + nonce.len());
    buf.extend_from_slice(SECRET_KEYWORD);
    buf.extend_from_slice(nonce);
    Md5::digest(&buf).into()
}

fn decrypt_body(body: &[u8]) -> Result<Vec<u8>, String> {
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

fn encrypt_body(plain: &[u8]) -> Vec<u8> {
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

// transform every gRPC frame payload in `data`.
// frames = [flag:1][length:4 BE][payload]
fn map_frames(
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

        // 0 len frames carry google.protobuf.Empty or hidden-field requests
        // pass them through unchanged so they get decoded anyways
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

#[derive(Debug, Clone, Default)]
pub struct QualiCryptLayer;

impl<S> tower::Layer<S> for QualiCryptLayer {
    type Service = QualiCryptService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        QualiCryptService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct QualiCryptService<S> {
    inner: S,
}

type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl<S, ResBody> tower::Service<http::Request<Body>> for QualiCryptService<S>
where
    S: tower::Service<http::Request<Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Error: Into<BoxError> + Send,
    S::Future: Send + 'static,
    ResBody: http_body::Body<Data = Bytes> + Send + 'static,
    ResBody::Error: Into<BoxError> + Send,
{
    type Response = http::Response<Body>;
    type Error = tonic::Status;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| tonic::Status::internal(format!("service not ready: {}", e.into())))
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let (parts, body) = req.into_parts();

            let collected = collect_body(body).await?;
            let body_bytes = collected.to_bytes();

            let plain = match map_frames(&body_bytes, decrypt_body) {
                Ok(plain) => plain,
                Err(e) => {
                    tracing::warn!(
                        "request decrypt failed: {e} (body {} bytes, head {:02x?})",
                        body_bytes.len(),
                        &body_bytes[..body_bytes.len().min(16)]
                    );
                    return Err(tonic::Status::internal(e));
                }
            };

            let req = http::Request::from_parts(
                parts,
                Body::new(http_body_util::Full::new(Bytes::from(plain))),
            );

            let resp = inner
                .call(req)
                .await
                .map_err(|e| tonic::Status::internal(format!("upstream: {}", e.into())))?;

            let (parts, body) = resp.into_parts();
            let collected = collect_body(body).await?;
            let trailers = collected.trailers().cloned();

            let enc = map_frames(&collected.to_bytes(), |p| Ok(encrypt_body(p)))
                .map_err(tonic::Status::internal)?;

            // forward the inner response's trailers (grpc-status etc.)
            // the client blocks on them after reading the body.
            let mut frames: Vec<Result<http_body::Frame<Bytes>, tonic::Status>> =
                vec![Ok(http_body::Frame::data(Bytes::from(enc)))];

            if let Some(trailers) = trailers {
                frames.push(Ok(http_body::Frame::trailers(trailers)));
            }

            let body = Body::new(http_body_util::StreamBody::new(
                tonic::codegen::tokio_stream::iter(frames),
            ));

            Ok(http::Response::from_parts(parts, body))
        })
    }
}

async fn collect_body<B>(body: B) -> Result<http_body_util::Collected<Bytes>, tonic::Status>
where
    B: http_body::Body<Data = Bytes>,
    B::Error: Into<BoxError>,
{
    use http_body_util::BodyExt;
    body.collect()
        .await
        .map_err(|e| tonic::Status::internal(format!("body: {}", e.into())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decrypts_known_uncompressed_vector() {
        // nonce = 0x11*16, plaintext = b"0123456789abcdefX" (17B, PKCS7 -> 32B)
        let body: Vec<u8> = [
            &(18u16).to_le_bytes()[..], // hdr_size = keylen+2 = 18
            &[0, 16],                   // compress=0, keylen=16
            &[0x11u8; 16],              // nonce
            &hex::decode_unchecked(
                "d993b2b0677090eabaca9b939166192b0b1ee848e864a234c4561e04440d0834",
            ),
        ]
        .concat();
        assert_eq!(decrypt_body(&body).unwrap(), b"0123456789abcdefX");
    }

    #[test]
    fn decrypts_known_zlib_vector() {
        // nonce = 0x22*16, plaintext = b"0123456789abcdef"*3, zlib-compressed
        let body: Vec<u8> = [
            &(18u16).to_le_bytes()[..],
            &[1, 16], // compress=1
            &[0x22u8; 16],
            &hex::decode_unchecked(
                "a753d6db4e5c14f4576c21bc2baeeb2681183e83889336a915dc53e9b63fe5c8",
            ),
        ]
        .concat();
        assert_eq!(
            decrypt_body(&body).unwrap(),
            b"0123456789abcdef0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let plain = b"hello world this is a protobuf-ish payload with some length";
        let body = encrypt_body(plain);
        assert_eq!(decrypt_body(&body).unwrap(), plain);
    }

    #[test]
    fn sniff_bin_survives_round_trip() {
        let original = crate::sniffs::AUTH_LOGIN_RESP;
        let body = encrypt_body(original);
        assert_eq!(decrypt_body(&body).unwrap(), original);
    }

    #[test]
    fn frame_mapping_round_trip() {
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

    #[tokio::test]
    async fn layer_decrypts_requests_and_encrypts_responses() {
        use http_body_util::BodyExt;
        use tower::{Layer as _, ServiceExt};

        let inner = tower::service_fn(|req: http::Request<Body>| async move {
            let collected = req
                .into_body()
                .collect()
                .await
                .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e))?;
            let bytes = collected.to_bytes();
            let mut trailers = http::HeaderMap::new();
            trailers.insert("grpc-status", http::HeaderValue::from_static("0"));
            let frames = vec![
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(http_body::Frame::data(bytes)),
                Ok(http_body::Frame::trailers(trailers)),
            ];
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(http::Response::new(Body::new(
                http_body_util::StreamBody::new(tonic::codegen::tokio_stream::iter(frames)),
            )))
        });

        let svc = QualiCryptLayer.layer(inner);

        let original = crate::sniffs::AUTH_LOGIN_RESP;
        let encrypted = encrypt_body(original);
        let mut req_frames = Vec::new();

        req_frames.push(0u8);
        req_frames.extend_from_slice(&(encrypted.len() as u32).to_be_bytes());
        req_frames.extend_from_slice(&encrypted);

        let resp = svc
            .oneshot(http::Request::new(Body::new(http_body_util::Full::new(
                Bytes::from(req_frames),
            ))))
            .await
            .unwrap();

        let collected = resp.into_body().collect().await.unwrap();
        let trailers = collected.trailers().cloned();
        let resp_bytes = collected.to_bytes();

        let plain = map_frames(&resp_bytes, decrypt_body).unwrap();
        assert_eq!(&plain[5..], original);

        let trailers = trailers.expect("trailers forwarded");
        assert_eq!(trailers.get("grpc-status").unwrap(), "0");
    }
}

#[cfg(test)]
mod hex {
    pub fn decode_unchecked(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
}
