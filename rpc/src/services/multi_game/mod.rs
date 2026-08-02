use tonic::{Request, Response, Status};

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

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct MultiGameService {}

#[tonic::async_trait]
impl MultiGame for MultiGameService {
    async fn list_ping_server(
        &self,
        _req: Request<()>,
    ) -> Result<Response<MultiGameListPingServerResponse>, Status> {
        Ok(Response::new(replay!(
            MultiGameListPingServerResponse,
            sniffs::MULTI_GAME_LIST_PING_SERVER_RESP
        )))
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
        Err(Status::unimplemented("MultiGame.list_invited_private_room"))
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
