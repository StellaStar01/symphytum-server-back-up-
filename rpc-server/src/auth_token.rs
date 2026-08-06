use tonic::{Request, Status};

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(B64[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(B64[n as usize & 63] as char);
        }
    }
    out
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for &c in s.as_bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return None,
        } as u32;
        buf = (buf << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}

pub fn mint(uid: &str) -> String {
    let header = r#"{"alg":"HS256","typ":"JWT"}"#;
    let now = database::unix_now();
    let payload = format!(
        r#"{{"sub":"{uid}","exp":{},"iat":{},"itn":"{}"}}"#,
        now + 86400,
        now,
        uid
    );
    format!(
        "{}.{}.{}",
        base64_encode(header.as_bytes()),
        base64_encode(payload.as_bytes()),
        base64_encode(&[0u8; 32])
    )
}

pub fn uid_from_token(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let bytes = base64_decode(payload)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value.get("sub")?.as_str().map(String::from)
}

pub fn uid<T>(req: &Request<T>) -> Result<String, Status> {
    let token = req
        .metadata()
        .get("x-app-auth-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Status::unauthenticated("missing x-app-auth-token"))?;
    uid_from_token(token).ok_or_else(|| Status::unauthenticated("invalid x-app-auth-token"))
}

pub fn uid_opt<T>(req: &Request<T>) -> Option<String> {
    let token = req
        .metadata()
        .get("x-app-auth-token")
        .and_then(|v| v.to_str().ok())?;
    uid_from_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_decode_roundtrip() {
        let token = mint("some-uid");
        assert_eq!(uid_from_token(&token).as_deref(), Some("some-uid"));
        assert_eq!(uid_from_token("garbage"), None);
    }
}
