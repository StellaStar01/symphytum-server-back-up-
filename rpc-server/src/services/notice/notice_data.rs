use resource::config::CONFIG;
use types::enums::NoticeType;
use types::rpc::api::NoticeInfo;

pub const NOTICE_IDS: [&str; 3] = ["notice-fake-001", "notice-repo-001", "notice-holodori-001"];

fn http_url(path: &str) -> String {
    format!(
        "http://{}:{}{}",
        CONFIG.http_server.host, CONFIG.http_server.port, path
    )
}

/// the fake notice catalog:
/// - `notice-fake-001`  -> "Welcome to Symphytum" (http-server welcome page)
/// - `notice-repo-001`  -> "Repository" (detail_url is the GitHub repo)
/// - `notice-holodori-001` -> "HolodoriDB" (detail_url is holodori.best)
pub fn notices() -> Vec<NoticeInfo> {
    vec![
        NoticeInfo {
            id: NOTICE_IDS[0].into(),
            title: "Welcome to Symphytum".into(),
            r#type: NoticeType::Information as i32,
            detail_url: http_url("/notice/notice-fake-001"),
            banner_asset_id: String::new(),
            is_read: false,
            start_time: 1_786_000_000_000,
            external_url: String::new(),
        },
        NoticeInfo {
            id: NOTICE_IDS[1].into(),
            title: "Repository".into(),
            r#type: NoticeType::Information as i32,
            detail_url: "https://github.com/yuvlian/symphytum-server".into(),
            banner_asset_id: String::new(),
            is_read: false,
            start_time: 1_786_000_000_000,
            external_url: String::new(),
        },
        NoticeInfo {
            id: NOTICE_IDS[2].into(),
            title: "HolodoriDB".into(),
            r#type: NoticeType::Information as i32,
            detail_url: "https://holodori.best/".into(),
            banner_asset_id: String::new(),
            is_read: false,
            start_time: 1_786_000_000_000,
            external_url: String::new(),
        },
    ]
}
