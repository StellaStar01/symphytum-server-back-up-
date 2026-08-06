use std::pin::Pin;
use std::task::{Context, Poll};

use tonic::body::Body;
use tonic::codegen::{Bytes, http};

use resource::quali_crypt::{decrypt_body, encrypt_body, map_frames};

type BoxFuture<T> = Pin<Box<dyn std::future::Future<Output = T> + Send>>;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

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

            // the client blocks on these trailers after reading the body
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
    use resource::quali_crypt::{decrypt_body, encrypt_body, map_frames};

    // arbitrary protobuf-shaped bytes for crypto round-trips
    static TEST_PAYLOAD: &[u8] = b"\x0a\x1dtest-symphytum-payload-0000000000000000";

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

        let original = TEST_PAYLOAD;
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
