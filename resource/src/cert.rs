use tonic::transport::{Identity, ServerTlsConfig};

use crate::config::CONFIG;

pub async fn get_tls_config() -> Result<ServerTlsConfig, std::io::Error> {
    let (cert, key) = tokio::join!(
        tokio::fs::read(CONFIG.cert_path()),
        tokio::fs::read(CONFIG.key_path()),
    );
    let identity = Identity::from_pem(cert?, key?);
    Ok(ServerTlsConfig::new().identity(identity))
}
