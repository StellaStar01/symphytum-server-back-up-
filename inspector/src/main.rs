use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;

use axum::Router;
use axum::extract::{Multipart, Query};
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::{get, post};

use resource::config::CONFIG;

use types::reflection::{DESCRIPTOR_POOL, decode_pretty};

const MAX_UPLOAD_BYTES: usize = 16 * 1024 * 1024;

const INDEX_HTML: &str = include_str!("../page/index.html");
const VIEW_HTML: &str = include_str!("../page/view.html");
const STYLE_CSS: &str = include_str!("../page/style.css");
const APP_JS: &str = include_str!("../page/app.js");

static SNIFF_CACHE: LazyLock<Mutex<HashMap<String, Sniff>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Req,
    Resp,
}

#[derive(Clone)]
struct Sniff {
    name: String,
    size: u64,
    direction: Option<Direction>,
    // proto route from the file name (e.g. `rpc.api.Auth_Login`);
    // none if not matching scheme
    route: Option<String>,
    modified: SystemTime,
    // message type short name (e.g. `AuthLoginResponse`) when the file name
    // resolves to a service method, else the file name; drives the name sort.
    sort_name: String,
    /// Ok(pretty) or Err(reason)
    decoded: Result<String, String>,
    hex: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    resource::config::init_tracing();
    let app = Router::new()
        .route("/", get(index))
        .route("/list", get(list))
        .route("/upload", post(upload))
        .route("/refresh", post(full_refresh))
        .route("/view", get(view));
    let bind = format!("{}:{}", CONFIG.inspector.host, CONFIG.inspector.port);
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("sniff inspector listening on http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<String> {
    let body = render_sniffs();
    Html(
        INDEX_HTML
            .replace("{CSS}", STYLE_CSS)
            .replace("{APP_JS}", APP_JS)
            .replace("{BODY}", &body),
    )
}

async fn list() -> Html<String> {
    Html(render_sniffs())
}

async fn view(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, (StatusCode, String)> {
    let name = params.get("file").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "missing \"file\" query param".to_string(),
        )
    })?;
    let sniff = scan_sniffs()
        .into_iter()
        .find(|s| &s.name == name)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("{name} is not in {}", CONFIG.sniffs_dir().display()),
            )
        })?;
    let (badge_class, badge_text) = match sniff.direction {
        Some(Direction::Req) => ("REQ", "REQ"),
        Some(Direction::Resp) => ("RESP", "RESP"),
        None => ("un", "?"),
    };
    let body = match &sniff.decoded {
        Ok(pretty) => escape_html(pretty),
        Err(err) => format!(
            "{}<br><br>bytes: {}",
            escape_html(err),
            escape_html(&sniff.hex)
        ),
    };
    let dmeta = if sniff.sort_name == sniff.name {
        human_size(sniff.size)
    } else {
        format!(
            "{} · {}",
            escape_html(&sniff.sort_name),
            human_size(sniff.size)
        )
    };
    let html = VIEW_HTML
        .replace("{CSS}", STYLE_CSS)
        .replace("{TITLE}", &escape_html(&sniff.name))
        .replace("{BADGE_CLASS}", badge_class)
        .replace("{BADGE_TEXT}", badge_text)
        .replace("{DMETA}", &dmeta)
        .replace("{BODY}", &body);
    Ok(Html(html))
}

async fn upload(mut multipart: Multipart) -> Result<Redirect, (StatusCode, String)> {
    let mut saved = 0usize;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() != Some("files") {
            continue;
        }
        let raw_name = field.file_name().unwrap_or("").to_string();
        let name = Path::new(&raw_name)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".bin") {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("only .bin files are accepted, got \"{raw_name}\""),
            ));
        }
        if name.starts_with('.') {
            return Err((
                StatusCode::BAD_REQUEST,
                "hidden files are not accepted".into(),
            ));
        }
        let data = field
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        if data.len() > MAX_UPLOAD_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("{name} exceeds {} bytes", MAX_UPLOAD_BYTES),
            ));
        }
        let dest = CONFIG.sniffs_dir().join(&name);
        tokio::fs::write(&dest, &data)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if let Ok(meta) = std::fs::metadata(&dest) {
            SNIFF_CACHE
                .lock()
                .unwrap()
                .insert(name.clone(), make_sniff(&name, &data, &meta));
        }
        tracing::info!("saved {} ({} bytes)", name, data.len());
        saved += 1;
    }
    tracing::info!("upload finished: {saved} file(s) saved");
    Ok(Redirect::to("/"))
}

async fn full_refresh() -> Html<String> {
    SNIFF_CACHE.lock().unwrap().clear();
    tracing::info!("cache cleared, re-decoding all sniffs");
    Html(render_sniffs())
}

fn render_sniffs() -> String {
    let mut sniffs = scan_sniffs();
    sniffs.sort_by(|a, b| b.modified.cmp(&a.modified));
    let mut html = String::new();
    for s in sniffs.iter() {
        let (badge_class, badge_text) = match s.direction {
            Some(Direction::Req) => ("REQ", "REQ"),
            Some(Direction::Resp) => ("RESP", "RESP"),
            None => ("un", "?"),
        };

        let body = match &s.decoded {
            Ok(pretty) => escape_html(pretty),
            Err(err) => format!("{}<br><br>bytes: {}", escape_html(err), escape_html(&s.hex)),
        };

        let dmeta = if s.sort_name == s.name {
            human_size(s.size)
        } else {
            format!("{} · {}", escape_html(&s.sort_name), human_size(s.size))
        };

        let href = format!("/view?file={}", escape_html(&s.name));
        let route_attr = s.route.as_deref().unwrap_or(&s.name);
        html.push_str(&format!(
            "<details data-file=\"{}\" data-name=\"{}\" data-route=\"{}\" data-ts=\"{}\"><summary><span class=\"badge {}\">{}</span><a class=\"fname\" href=\"{}\">{}</a><span class=\"dmeta\">{}</span></summary><pre>{}</pre><div class=\"prefoot\"><a class=\"view-link\" href=\"{}\" target=\"_blank\" rel=\"noopener\">open in new tab</a></div></details>\n",
            escape_html(&s.name),
            escape_html(&s.sort_name),
            escape_html(route_attr),
            mtime_millis(&s.modified),
            badge_class,
            badge_text,
            href,
            escape_html(&s.name),
            dmeta,
            body,
            href,
        ));
    }
    if sniffs.is_empty() {
        html.push_str("<p class=\"meta\">no .bin files yet — upload some above, or drop them into sniffs/</p>");
    }
    html
}

fn mtime_millis(t: &SystemTime) -> u64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn human_size(n: u64) -> String {
    if n >= 1024 * 1024 {
        format!("{:.1} MB", n as f64 / (1024.0 * 1024.0))
    } else if n >= 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{n} B")
    }
}

fn make_sniff(name: &str, bytes: &[u8], meta: &std::fs::Metadata) -> Sniff {
    let parsed = parse_name(name);
    Sniff {
        modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        size: meta.len(),
        hex: hex_preview(bytes),
        direction: parsed.as_ref().map(|(_, d)| *d),
        route: parsed.as_ref().map(|(r, _)| r.clone()),
        sort_name: parsed
            .as_ref()
            .and_then(|(route, direction)| resolve(route, *direction))
            .map(|full_name| {
                full_name
                    .rsplit('.')
                    .next()
                    .unwrap_or(&full_name)
                    .to_string()
            })
            .unwrap_or_else(|| name.to_string()),
        decoded: decode_sniff(name, bytes),
        name: name.to_string(),
    }
}

fn scan_sniffs() -> Vec<Sniff> {
    let mut cache = SNIFF_CACHE.lock().unwrap();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let sniffs_dir = CONFIG.sniffs_dir();
    let entries = match std::fs::read_dir(&sniffs_dir) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("cannot read {}: {e}", sniffs_dir.display());
            return out;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("bin") {
            continue;
        }
        let name = match path.file_name().and_then(|s| s.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        seen.insert(name.clone());
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("metadata {name}: {e}");
                continue;
            }
        };
        let (mtime, size) = (
            meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            meta.len(),
        );
        if let Some(cached) = cache.get(&name) {
            if cached.modified == mtime && cached.size == size {
                out.push(cached.clone());
                continue;
            }
        }
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!("read {name}: {e}");
                continue;
            }
        };
        let sniff = make_sniff(&name, &bytes, &meta);
        cache.insert(name.clone(), sniff.clone());
        out.push(sniff);
    }
    cache.retain(|name, _| seen.contains(name));
    out
}

// `<route>_<YYYYMMDD>_<HHMMSS>_<FFF>_<REQ|RESP>_<INDEX>.bin` -> (route, direction).
// Parsed from the end: last token is the index (digits), previous is REQ|RESP,
// the three before that are the timestamp. Anything else is rejected.
fn parse_name(name: &str) -> Option<(String, Direction)> {
    let stem = name.strip_suffix(".bin")?;
    let parts: Vec<&str> = stem.split('_').collect();
    if parts.len() < 6 {
        return None;
    }
    if !parts[parts.len() - 1].bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let direction = match parts[parts.len() - 2] {
        "REQ" => Direction::Req,
        "RESP" => Direction::Resp,
        _ => return None,
    };
    let ts = &parts[parts.len() - 5..parts.len() - 2];
    if !ts
        .iter()
        .all(|t| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    let route = parts[..parts.len() - 5].join("_");
    if route.is_empty() {
        return None;
    }
    Some((route, direction))
}

// `rpc.api.Auth_Login` + REQ -> the full message name of Auth.Login's input
// (`rpc.api.AuthLoginRequest`). Tries every `_` split point right-to-left.
fn resolve(route: &str, direction: Direction) -> Option<String> {
    for i in (0..route.len()).rev() {
        if !route.is_char_boundary(i) || route.as_bytes()[i] != b'_' {
            continue;
        }
        let service = &route[..i];
        let method = &route[i + 1..];
        let Some(svc) = DESCRIPTOR_POOL.get_service_by_name(service) else {
            continue;
        };
        let Some(m) = svc.methods().find(|m| m.name() == method) else {
            continue;
        };
        return Some(match direction {
            Direction::Req => m.input().full_name().to_owned(),
            Direction::Resp => m.output().full_name().to_owned(),
        });
    }
    None
}

fn decode_sniff(name: &str, bytes: &[u8]) -> Result<String, String> {
    let (route, direction) = parse_name(name).ok_or_else(|| {
        "filename does not match <route>_<YYYYMMDD>_<HHMMSS>_<FFF>_<REQ|RESP>_<INDEX>.bin"
            .to_string()
    })?;
    let full_name = resolve(&route, direction)
        .ok_or_else(|| format!("no service method matches route \"{route}\""))?;
    decode_pretty(&full_name, bytes).map_err(|e| format!("decode {full_name}: {e}"))
}

fn hex_preview(bytes: &[u8]) -> String {
    let shown = bytes
        .iter()
        .take(64)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    if bytes.len() > 64 {
        format!("{shown} ... ({} bytes total)", bytes.len())
    } else {
        shown
    }
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
