use database::Database;
use tonic::{Request, Response, Status};

use types::rpc::api::multi_game_list_ping_server_response::PingServer;
use types::rpc::api::multi_game_server::MultiGame;
use types::rpc::api::{
    MultiGameCancelMatchRequest, MultiGameCancelMatchResponse,
    MultiGameCheckInvitedPrivateRoomResponse, MultiGameCheckMatchStatusRequest,
    MultiGameCheckMatchStatusResponse, MultiGameCheckPrivateRoomRematchStatusRequest,
    MultiGameCheckPrivateRoomRematchStatusResponse, MultiGameEnterPrivateRoomRequest,
    MultiGameEnterPrivateRoomResponse, MultiGameGeneratePrivateRoomIdResponse,
    MultiGameGetPrivateRoomInfoRequest, MultiGameGetPrivateRoomInfoResponse,
    MultiGameInviteAllFriendToPrivateRoomRequest, MultiGameInviteAllFriendToPrivateRoomResponse,
    MultiGameInviteFriendToPrivateRoomRequest, MultiGameInviteFriendToPrivateRoomResponse,
    MultiGameListBasicUserInfoRequest, MultiGameListBasicUserInfoResponse,
    MultiGameListInvitedPrivateRoomResponse, MultiGameListPingServerResponse,
    MultiGameListPrivateRoomFriendUserRequest, MultiGameListPrivateRoomFriendUserResponse,
    MultiGameSaveReactionSendInfoRequest, MultiGameSaveReactionSendInfoResponse,
    MultiGameSetPrivateRoomRematchStatusRequest, MultiGameSetPrivateRoomRematchStatusResponse,
};

#[derive(Clone)]
pub struct MultiGameService {
    _db: Database,
}

impl MultiGameService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { _db: db })
    }
}

/// Will never be implemented cuz i have no friends
#[tonic::async_trait]
impl MultiGame for MultiGameService {
    async fn list_ping_server(
        &self,
        _req: Request<()>,
    ) -> Result<Response<MultiGameListPingServerResponse>, Status> {
        let servers = [
            ("us", "136.116.137.91"),
            ("us", "34.173.34.214"),
            ("eu", "35.205.108.128"),
            ("eu", "34.52.244.151"),
            ("tw", "34.81.140.104"),
            ("tw", "34.80.168.26"),
            ("jp", "35.200.112.201"),
            ("jp", "34.84.97.216"),
            ("sg", "34.142.191.35"),
            ("sg", "34.158.58.114"),
        ]
        .into_iter()
        .map(|(region, endpoint)| PingServer {
            region: region.into(),
            endpoint: endpoint.into(),
        })
        .collect();
        Ok(Response::new(MultiGameListPingServerResponse {
            servers,
            common_response: None,
        }))
    }
    async fn generate_private_room_id(
        &self,
        _req: Request<()>,
    ) -> Result<Response<MultiGameGeneratePrivateRoomIdResponse>, Status> {
        Err(Status::unimplemented("MultiGame.generate_private_room_id"))
    }

    async fn get_private_room_info(
        &self,
        _req: Request<MultiGameGetPrivateRoomInfoRequest>,
    ) -> Result<Response<MultiGameGetPrivateRoomInfoResponse>, Status> {
        Err(Status::unimplemented("MultiGame.get_private_room_info"))
    }

    async fn enter_private_room(
        &self,
        _req: Request<MultiGameEnterPrivateRoomRequest>,
    ) -> Result<Response<MultiGameEnterPrivateRoomResponse>, Status> {
        Err(Status::unimplemented("MultiGame.enter_private_room"))
    }

    async fn invite_friend_to_private_room(
        &self,
        _req: Request<MultiGameInviteFriendToPrivateRoomRequest>,
    ) -> Result<Response<MultiGameInviteFriendToPrivateRoomResponse>, Status> {
        Err(Status::unimplemented(
            "MultiGame.invite_friend_to_private_room",
        ))
    }

    async fn invite_all_friend_to_private_room(
        &self,
        _req: Request<MultiGameInviteAllFriendToPrivateRoomRequest>,
    ) -> Result<Response<MultiGameInviteAllFriendToPrivateRoomResponse>, Status> {
        Err(Status::unimplemented(
            "MultiGame.invite_all_friend_to_private_room",
        ))
    }

    async fn list_private_room_friend_user(
        &self,
        _req: Request<MultiGameListPrivateRoomFriendUserRequest>,
    ) -> Result<Response<MultiGameListPrivateRoomFriendUserResponse>, Status> {
        Err(Status::unimplemented(
            "MultiGame.list_private_room_friend_user",
        ))
    }

    async fn check_invited_private_room(
        &self,
        _req: Request<()>,
    ) -> Result<Response<MultiGameCheckInvitedPrivateRoomResponse>, Status> {
        Err(Status::unimplemented(
            "MultiGame.check_invited_private_room",
        ))
    }

    async fn list_invited_private_room(
        &self,
        _req: Request<()>,
    ) -> Result<Response<MultiGameListInvitedPrivateRoomResponse>, Status> {
        Ok(Response::new(
            MultiGameListInvitedPrivateRoomResponse::default(),
        ))
    }

    async fn check_match_status(
        &self,
        _req: Request<MultiGameCheckMatchStatusRequest>,
    ) -> Result<Response<MultiGameCheckMatchStatusResponse>, Status> {
        Err(Status::unimplemented("MultiGame.check_match_status"))
    }

    async fn cancel_match(
        &self,
        _req: Request<MultiGameCancelMatchRequest>,
    ) -> Result<Response<MultiGameCancelMatchResponse>, Status> {
        Err(Status::unimplemented("MultiGame.cancel_match"))
    }

    async fn set_private_room_rematch_status(
        &self,
        _req: Request<MultiGameSetPrivateRoomRematchStatusRequest>,
    ) -> Result<Response<MultiGameSetPrivateRoomRematchStatusResponse>, Status> {
        Err(Status::unimplemented(
            "MultiGame.set_private_room_rematch_status",
        ))
    }

    async fn check_private_room_rematch_status(
        &self,
        _req: Request<MultiGameCheckPrivateRoomRematchStatusRequest>,
    ) -> Result<Response<MultiGameCheckPrivateRoomRematchStatusResponse>, Status> {
        Err(Status::unimplemented(
            "MultiGame.check_private_room_rematch_status",
        ))
    }

    async fn list_basic_user_info(
        &self,
        _req: Request<MultiGameListBasicUserInfoRequest>,
    ) -> Result<Response<MultiGameListBasicUserInfoResponse>, Status> {
        Err(Status::unimplemented("MultiGame.list_basic_user_info"))
    }

    async fn save_reaction_send_info(
        &self,
        _req: Request<MultiGameSaveReactionSendInfoRequest>,
    ) -> Result<Response<MultiGameSaveReactionSendInfoResponse>, Status> {
        Err(Status::unimplemented("MultiGame.save_reaction_send_info"))
    }
}
