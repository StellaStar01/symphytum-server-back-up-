use tonic::{Request, Response, Status};
use tonic_middleware::MiddlewareLayer;
use types::rpc::api::common::{Response as CommonResponse, UserData};
use types::rpc::api::user_server::{User, UserServer};
use types::rpc::api::{UserDeleteResponse, UserGetResponse};

mod middleware;
use middleware::QualiCrypt;

#[derive(Default)]
pub struct MyUser {}

#[tonic::async_trait]
impl User for MyUser {
    async fn get(&self, _request: Request<()>) -> Result<Response<UserGetResponse>, Status> {
        let user_data = UserData {
            ..Default::default()
        };
        let response = UserGetResponse {
            user_data: Some(user_data),
        };
        Ok(Response::new(response))
    }

    async fn delete(&self, _request: Request<()>) -> Result<Response<UserDeleteResponse>, Status> {
        let response = UserDeleteResponse {
            common_response: Some(CommonResponse {
                ..Default::default()
            }),
        };
        Ok(Response::new(response))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3000".parse()?;
    let user = MyUser::default();

    println!("gRPC Server listening on {}", addr);

    tonic::transport::Server::builder()
        .layer(MiddlewareLayer::new(QualiCrypt))
        .add_service(UserServer::new(user))
        .serve(addr)
        .await?;

    Ok(())
}
