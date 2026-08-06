use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, Response};
use axum::routing::get;

use resource::config::{CONFIG, init_tracing};

/// placeholder img
const PALETTE_JPG: &[u8] = include_bytes!("../dasli.jpg");

fn jpeg_response() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"))
        .body(Body::from(PALETTE_JPG))
        .expect("static body")
}

async fn palette(Path(_number): Path<String>) -> Response {
    jpeg_response()
}

async fn palette_upload_get(Path(_token): Path<String>) -> Response {
    jpeg_response()
}

async fn palette_upload_put(Path(_token): Path<String>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .expect("ok")
}

async fn notice(Path(notice_id): Path<String>) -> Html<String> {
    Html(format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{notice_id}</title></head>\
         <body style=\"font-family:sans-serif;text-align:center;padding-top:4rem\">\
         <h1>Welcome to Symphytum</h1>\
         <p>Notice: {notice_id}</p>\
         </body></html>"
    ))
}

async fn index() -> Html<String> {
    Html(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>hattp</title></head>\
         <body style=\"font-family:monospace\">\
         <h1>hattp</h1>\
         <ul><li><a href=\"/palette/1\">/palette/1</a> (fake palette image)</li>\
         <li><a href=\"/notice/notice-fake-001\">/notice/notice-fake-001</a> (fake notice)</li></ul>\
         </body></html>"
        .into(),
    )
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let addr = format!("{}:{}", CONFIG.http_server.host, CONFIG.http_server.port);
    let app = Router::new()
        .route("/", get(index))
        .route("/palette/{number}", get(palette))
        .route(
            "/palette_upload/{token}",
            get(palette_upload_get).put(palette_upload_put),
        )
        .route("/notice/{notice_id}", get(notice));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("serving http-server on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
