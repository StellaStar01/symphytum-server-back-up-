use database::Database;
use tonic::{Request, Response, Status};
use types::rpc::api::UserContentCdnGetSignedCookieResponse;
use types::rpc::api::common::UserContentCdnSignedCookie;
use types::rpc::api::user_content_cdn_server::UserContentCdn;

#[derive(Clone)]
pub struct UserContentCdnService {
    _db: Database,
}

impl UserContentCdnService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

#[tonic::async_trait]
impl UserContentCdn for UserContentCdnService {
    async fn get_signed_cookie(
        &self,
        _req: Request<()>,
    ) -> Result<Response<UserContentCdnGetSignedCookieResponse>, Status> {
        // the client only downloads user-content images when this RPC succeeds
        Ok(Response::new(UserContentCdnGetSignedCookieResponse {
            signed_cookie: Some(UserContentCdnSignedCookie {
                policy: "URLPrefix=aHR0cHM6Ly91c2VyLWNvbnRlbnQuZ2FtZS1ob2xvbGl2ZS1kcmVhbXMuY29tLw==:Expires=1785653715:KeyName=dreams-user-content-cdn-signed-url-key".into(),
                signature: "YjtHzkK2wC1ulBx-8nlVtzn88vo".into(),
                key_name: "dreams-user-content-cdn-signed-url-key".into(),
                expired_time_milliseconds: 1785653715490,
            }),
        }))
    }
}
