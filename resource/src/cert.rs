use tonic::transport::{Identity, ServerTlsConfig};

const CERT_PEM: &[u8] = include_bytes!("../cert/cert.pem");
const KEY_PEM: &[u8] = include_bytes!("../cert/key.pem");

pub fn get_tls_config() -> ServerTlsConfig {
    let identity = Identity::from_pem(CERT_PEM, KEY_PEM);
    ServerTlsConfig::new().identity(identity)
}
