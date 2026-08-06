use std::path::Path;

use prost::Message;
use prost_reflect::DynamicMessage;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use tonic::codegen::{Bytes, http};
use types::common::MasterTagPack;
use types::reflection::DESCRIPTOR_POOL;
use types::rpc::api::MasterGetResponse;
use types::rpc::api::master_client::MasterClient;

use resource::config::repo_root;
use resource::quali_crypt::decrypt_body;
use resource::sql_cipher::decrypt_file;

const ENDPOINT: &str = "https://as.game-hololive-dreams.com";
const APP_VERSION: &str = "1.0.101";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let master_dir = repo_root().join("resource").join("master");
    tokio::fs::create_dir_all(&master_dir).await?;

    let resp = fetch_master_get().await?;
    let version = resp.version.clone();

    // only dump when the remote version changed
    let version_path = master_dir.join("version.txt");
    if let Ok(old) = tokio::fs::read_to_string(&version_path).await {
        if old.trim() == version {
            println!("master version unchanged ({version}); keeping existing data");
            return Ok(());
        }
    }

    println!(
        "master version {version} -> dumping {} tables",
        resp.master_tag_packs.len()
    );
    tokio::fs::write(master_dir.join("master_get.bin"), resp.encode_to_vec()).await?;

    for pack in &resp.master_tag_packs {
        dump_table(&master_dir, pack).await?;
    }

    tokio::fs::write(version_path, version).await?;
    println!("done");
    Ok(())
}

#[derive(Clone, Default)]
struct DecryptResponseLayer;

#[derive(Clone)]
struct DecryptResponseService<S>(S);

impl<S> tower::Layer<S> for DecryptResponseLayer {
    type Service = DecryptResponseService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        DecryptResponseService(inner)
    }
}

impl<S, ResBody> tower::Service<http::Request<tonic::body::Body>> for DecryptResponseService<S>
where
    S: tower::Service<http::Request<tonic::body::Body>, Response = http::Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
    ResBody: http_body::Body<Data = Bytes> + Send + 'static + std::fmt::Debug,
    ResBody::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = http::Response<tonic::body::Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut req: http::Request<tonic::body::Body>) -> Self::Future {
        let mut inner = self.0.clone();
        Box::pin(async move {
            for (k, v) in [
                (
                    "user-agent",
                    "grpc-dotnet/2.76.0 (Mono Unity; CLR 4.0.30319.17020; netstandard2.1; windows; x64)",
                ),
                ("te", "trailers"),
                ("grpc-accept-encoding", "identity"),
                ("x-app-version", APP_VERSION),
                (
                    "x-app-device-name",
                    "ASUS Vivobook S 14 S3407CA_S3407CA (ASUSTeK COMPUTER INC.)",
                ),
                ("x-app-os-version", "Windows 11 (10.0.26100)"),
                ("x-app-lang-type", "eng"),
                ("x-app-os-type", "Windows"),
                ("x-app-store-type", "Steam"),
                ("x-app-bundle-id", "game.qualiarts.hololive.dreams.com"),
                ("x-app-cpu", "Intel(R) Core(TM) Ultra 7 255H"),
                ("x-app-gpu", "Intel(R) Graphics"),
                ("x-app-operating-system-family", "Windows"),
                ("x-app-device-type", "Desktop"),
                ("x-app-graphics-device-type", "Direct3D11"),
                (
                    "x-app-graphics-device-version",
                    "Direct3D 11.0 [level 11.1]",
                ),
                ("x-app-graphics-device-vendor-id", "32902"),
                ("x-app-graphics-device-memory-size", "9009"),
                ("x-app-graphics-shader-level", "50"),
                ("x-app-processor-model", "Intel(R) Core(TM) Ultra 7 255H"),
                ("x-app-processor-count", "16"),
                ("x-app-system-memory-size", "15806"),
                ("x-app-screen-dpi", "96"),
                ("x-app-screen-height", "943"),
                ("x-app-screen-width", "1676"),
                ("grpc-timeout", "10S"),
                ("content-type", "application/grpc+proto-enc"),
            ] {
                let h = req.headers_mut();
                h.remove(k);
                h.insert(k, http::HeaderValue::from_static(v));
            }

            use tower::ServiceExt;
            let resp = inner.ready().await?.call(req).await?;
            let (parts, body) = resp.into_parts();
            let bytes = http_body_util::BodyExt::collect(body).await?.to_bytes();

            // strip gRPC frame headers, decrypt the single envelope
            let payloads = frame_payloads(&bytes);
            if payloads.is_empty() {
                panic!(
                    "payloads is empty, maybe APP_VERSION {} is outdated.",
                    APP_VERSION
                );
            }
            let plain = decrypt_body(&payloads)?;
            let mut framed = Vec::with_capacity(plain.len() + 5);
            framed.push(0u8);
            framed.extend_from_slice(&(plain.len() as u32).to_be_bytes());
            framed.extend_from_slice(&plain);

            let mut trailers = http::HeaderMap::new();
            trailers.insert("grpc-status", http::HeaderValue::from_static("0"));
            let body = tonic::body::Body::new(http_body_util::StreamBody::new(
                tonic::codegen::tokio_stream::iter(vec![
                    Ok::<_, Box<dyn std::error::Error + Send + Sync>>(http_body::Frame::data(
                        Bytes::from(framed),
                    )),
                    Ok(http_body::Frame::trailers(trailers)),
                ]),
            ));
            Ok(http::Response::from_parts(parts, body))
        })
    }
}

fn frame_payloads(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 5 <= data.len() {
        let len = u32::from_be_bytes(data[pos + 1..pos + 5].try_into().unwrap()) as usize;
        pos += 5;
        if pos + len > data.len() {
            break;
        }
        out.extend_from_slice(&data[pos..pos + len]);
        pos += len;
    }
    out
}

async fn fetch_master_get() -> Result<MasterGetResponse, Box<dyn std::error::Error>> {
    let tls = tonic::transport::ClientTlsConfig::new().with_webpki_roots();
    let channel = tonic::transport::Endpoint::from_shared(ENDPOINT)?
        .tls_config(tls)?
        .connect()
        .await?;
    let svc = tower::ServiceBuilder::new()
        .layer(DecryptResponseLayer)
        .service(channel);
    let mut client = MasterClient::new(svc);
    let resp = client.get(()).await?;
    Ok(resp.into_inner())
}

async fn dump_table(
    master_dir: &Path,
    pack: &MasterTagPack,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  {} ({})", pack.r#type, pack.file_size);
    let body = reqwest::get(&pack.download_url).await?.bytes().await?;
    let plain = decrypt_file(&body, &pack.crypto_key)?;

    let tmp = master_dir.join(format!(".tmp-{}-{}.db", std::process::id(), pack.file_name));
    tokio::fs::write(&tmp, &plain).await?;
    let rows = read_table(&tmp, &pack).await?;
    let _ = tokio::fs::remove_file(&tmp).await;

    write_json_files(master_dir, pack, rows).await?;
    Ok(())
}

async fn read_table(
    path: &Path,
    pack: &MasterTagPack,
) -> Result<Vec<(String, Vec<u8>)>, Box<dyn std::error::Error>> {
    let table = table_name(&pack.r#type);
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Off);
    let pool = SqlitePool::connect_with(opts).await?;

    // column layout (PRAGMA table_info does not take bound params)
    let cols: Vec<(i64, String, String)> =
        sqlx::query_as(&format!("PRAGMA table_info(\"{table}\")"))
            .fetch_all(&pool)
            .await?
            .into_iter()
            .map(|(cid, name, kind): (i64, String, String)| (cid, name, kind))
            .collect();
    let data_col = cols.iter().position(|(_, n, _)| n == "data");
    eprintln!(
        "  cols: {:?}",
        cols.iter().map(|(_, n, _)| n.clone()).collect::<Vec<_>>()
    );

    let mut out = Vec::new();
    if let Some(di) = data_col {
        let sql = format!("SELECT * FROM \"{table}\"");
        use sqlx::Row;
        for row in sqlx::query(&sql).fetch_all(&pool).await? {
            if let Ok(blob) = row.try_get::<Vec<u8>, _>(di) {
                let key = cols
                    .iter()
                    .enumerate()
                    .filter(|(i, (_, _, _))| *i != di)
                    .map(|(i, (_, name, _))| {
                        let v = row
                            .try_get::<String, _>(i)
                            .or_else(|_| row.try_get::<i64, _>(i).map(|n| n.to_string()))
                            .unwrap_or_default();
                        format!("{name}={v}")
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                out.push((key, blob));
            }
        }
    }
    pool.close().await;
    Ok(out)
}

async fn write_if_changed(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let unchanged = tokio::fs::read_to_string(path)
        .await
        .map(|old| old == content)
        .unwrap_or(false);
    if unchanged {
        return Ok(());
    }
    tokio::fs::write(path, content).await?;
    Ok(())
}

fn table_name(type_name: &str) -> String {
    if type_name.starts_with("LangClient") {
        "LangClient".into()
    } else if type_name.starts_with("Lang") {
        "Lang".into()
    } else {
        type_name.into()
    }
}

async fn write_json_files(
    master_dir: &Path,
    pack: &MasterTagPack,
    rows: Vec<(String, Vec<u8>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let type_name = pack.r#type.as_str();
    // Lang packs are per-(table, locale); rows use the generic Lang / LangClient message.
    let desc_name = if type_name.starts_with("LangClient") {
        "entity.master.LangClient"
    } else if type_name.starts_with("Lang") {
        "entity.master.Lang"
    } else {
        return dump_plain(master_dir, type_name, &rows).await;
    };
    let Some(desc) = DESCRIPTOR_POOL.get_message_by_name(desc_name) else {
        eprintln!("  !! no proto for {type_name}; skipping");
        return Ok(());
    };
    let mut values = Vec::with_capacity(rows.len());
    for (_key, blob) in &rows {
        let msg = DynamicMessage::decode(desc.clone(), blob.as_slice())
            .map_err(|e| format!("row decode {}: {e}", type_name))?;
        values.push(serde_json::to_value(&msg)?);
    }
    write_if_changed(
        &master_dir.join(format!("{type_name}.json")),
        &serde_json::to_string(&values)?,
    )
    .await?;
    Ok(())
}

async fn dump_plain(
    master_dir: &Path,
    type_name: &str,
    rows: &[(String, Vec<u8>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(desc) = DESCRIPTOR_POOL.get_message_by_name(&format!("entity.master.{type_name}"))
    else {
        eprintln!("  !! no proto for {type_name}; skipping (client fetches it itself)");
        return Ok(());
    };
    let mut values = Vec::with_capacity(rows.len());
    for (_key, blob) in rows {
        let msg = DynamicMessage::decode(desc.clone(), blob.as_slice())
            .map_err(|e| format!("row decode {}: {e}", type_name))?;
        values.push(serde_json::to_value(&msg)?);
    }
    write_if_changed(
        &master_dir.join(format!("{type_name}.json")),
        &serde_json::to_string(&values)?,
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn write_if_changed_skips_identical_content() {
        let dir = std::env::temp_dir().join(format!("symmaster-{}", std::process::id()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let path = dir.join("t.json");
        write_if_changed(&path, "abc").await.unwrap();
        assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "abc");
        // identical content: no rewrite
        write_if_changed(&path, "abc").await.unwrap();
        assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "abc");
        // changed content: rewritten
        write_if_changed(&path, "abd").await.unwrap();
        assert_eq!(tokio::fs::read_to_string(&path).await.unwrap(), "abd");
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
