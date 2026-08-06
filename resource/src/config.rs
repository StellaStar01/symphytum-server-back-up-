use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::Deserialize;

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);

#[derive(Debug, Clone, Deserialize)]
pub struct RpcServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InspectorConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_sniffs_path")]
    pub sniffs_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CertConfig {
    #[serde(default = "default_cert_file")]
    pub cert_file_path: String,
    #[serde(default = "default_key_file")]
    pub key_file_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogLevelConfig {
    #[serde(default)]
    pub env_filters: Vec<String>,
    #[serde(default)]
    pub log_body: bool,
    #[serde(default)]
    pub log_header: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: LogLevel,
    #[serde(default = "default_debug_log")]
    pub debug: LogLevelConfig,
    #[serde(default = "default_info_log")]
    pub info: LogLevelConfig,
}

impl LogConfig {
    /// The config for the currently selected level.
    pub fn current(&self) -> &LogLevelConfig {
        match self.level {
            LogLevel::Debug => &self.debug,
            LogLevel::Info => &self.info,
        }
    }

    /// The tracing env filter: the selected level's entries joined with ",".
    pub fn env_filter(&self) -> String {
        self.current().env_filters.join(",")
    }

    pub fn log_bodies(&self) -> bool {
        self.current().log_body
    }

    pub fn log_headers(&self) -> bool {
        self.current().log_header
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_rpc", rename = "rpc-server")]
    pub rpc_server: RpcServerConfig,
    #[serde(default = "default_inspector")]
    pub inspector: InspectorConfig,
    #[serde(default = "default_cert")]
    pub cert: CertConfig,
    #[serde(default = "default_log")]
    pub log: LogConfig,
    #[serde(default = "default_http_server", rename = "http-server")]
    pub http_server: HttpServerConfig,
}

fn default_rpc() -> RpcServerConfig {
    RpcServerConfig {
        host: "127.0.0.1".into(),
        port: 3000,
    }
}

fn default_inspector() -> InspectorConfig {
    InspectorConfig {
        host: "127.0.0.1".into(),
        port: 3001,
        sniffs_path: default_sniffs_path(),
    }
}

fn default_cert() -> CertConfig {
    CertConfig {
        cert_file_path: default_cert_file(),
        key_file_path: default_key_file(),
    }
}

fn default_http_server() -> HttpServerConfig {
    HttpServerConfig {
        host: "127.0.0.1".into(),
        port: 3010,
    }
}

fn default_log() -> LogConfig {
    LogConfig {
        level: default_log_level(),
        debug: default_debug_log(),
        info: default_info_log(),
    }
}

fn default_log_level() -> LogLevel {
    LogLevel::Debug
}

fn default_debug_log() -> LogLevelConfig {
    LogLevelConfig {
        env_filters: vec!["debug".into(), "tonic_debug=debug".into()],
        log_body: true,
        log_header: false,
    }
}

fn default_info_log() -> LogLevelConfig {
    LogLevelConfig {
        env_filters: vec!["info".into(), "tonic_debug=info".into()],
        log_body: false,
        log_header: false,
    }
}

fn default_cert_file() -> String {
    "resource/cert/cert.pem".into()
}

fn default_key_file() -> String {
    "resource/cert/key.pem".into()
}

fn default_sniffs_path() -> String {
    "inspector/sniffs".into()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rpc_server: default_rpc(),
            inspector: default_inspector(),
            cert: default_cert(),
            log: default_log(),
            http_server: default_http_server(),
        }
    }
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("resource manifest dir has no parent")
        .to_path_buf()
}

pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(CONFIG.log.env_filter())
        .try_init();
}

pub fn debug_layer() -> tonic_debug::DebugLayer {
    tonic_debug::DebugLayer::new()
        .log_headers(CONFIG.log.log_headers())
        .log_bodies(CONFIG.log.log_bodies())
}

impl Config {
    pub fn load() -> Config {
        let candidates = [
            PathBuf::from("config.toml"),
            repo_root().join("config.toml"),
        ];
        for path in &candidates {
            match std::fs::read_to_string(path) {
                Ok(text) => match toml::from_str(&text) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("config: {}: bad toml: {e}", path.display()),
                },
                Err(_) => continue,
            }
        }
        let looked: Vec<String> = candidates.iter().map(|p| p.display().to_string()).collect();
        eprintln!("config: no config.toml found in {looked:?}; using defaults");
        Config::default()
    }

    fn absolute(&self, p: &str) -> PathBuf {
        let b = PathBuf::from(p);
        if b.is_absolute() {
            b
        } else {
            repo_root().join(b)
        }
    }

    pub fn cert_path(&self) -> PathBuf {
        self.absolute(&self.cert.cert_file_path)
    }
    pub fn key_path(&self) -> PathBuf {
        self.absolute(&self.cert.key_file_path)
    }
    pub fn sniffs_dir(&self) -> PathBuf {
        self.absolute(&self.inspector.sniffs_path)
    }
}
