macro_rules! sniff {
    ($file:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../inspector/sniffs/",
            $file
        ))
    };
}

pub const SYSTEM_GET_SYSTEM_INFO_RESP: &[u8] =
    sniff!("rpc.api.System_GetSystemInfo_20260801_135432_783_RESP_0.bin");

pub const MASTER_GET_RESP: &[u8] = sniff!("rpc.api.Master_Get_20260801_135434_203_RESP_0.bin");

pub const AUTH_LOGIN_RESP: &[u8] = sniff!("rpc.api.Auth_Login_20260801_135434_968_RESP_0.bin");

pub const USER_GET_RESP: &[u8] = sniff!("rpc.api.User_Get_20260801_135435_258_RESP_0.bin");

pub const HOME_LOGIN_RESP: &[u8] = sniff!("rpc.api.Home_Login_20260801_135514_571_RESP_0.bin");

pub const MULTI_GAME_LIST_PING_SERVER_RESP: &[u8] =
    sniff!("rpc.api.MultiGame_ListPingServer_20260801_135515_082_RESP_0.bin");

pub const PARK_ENTER_RESP: &[u8] = sniff!("rpc.api.Park_Enter_20260801_135516_445_RESP_0.bin");

pub const EVENT_LIST_EVENT_INFO_RESP: &[u8] =
    sniff!("rpc.api.Event_ListEventInfo_20260801_135516_668_RESP_0.bin");

pub const PARK_REFRESH_RESP: &[u8] = sniff!("rpc.api.Park_Refresh_20260801_135517_102_RESP_0.bin");

pub const NOTIFICATION_LIST_RESP: &[u8] =
    sniff!("rpc.api.Notification_List_20260801_135518_347_RESP_0.bin");

pub const STARTUP_NOTIFICATION_READ_RESP: &[u8] =
    sniff!("rpc.api.StartupNotification_Read_20260801_135523_266_RESP_0.bin");

pub const STARTUP_NOTIFICATION_READ_MUSIC_RESP: &[u8] =
    sniff!("rpc.api.StartupNotification_ReadMusic_20260801_135533_581_RESP_0.bin");

pub const LOGIN_BONUS_CHECK_RESP: &[u8] =
    sniff!("rpc.api.LoginBonus_Check_20260801_140457_139_RESP_0.bin");

pub const GACHA_LIST_RESP: &[u8] = sniff!("rpc.api.Gacha_List_20260801_135518_631_RESP_0.bin");

pub const EXCHANGE_LIST_GACHA_PICKUP_260728_RESP: &[u8] =
    sniff!("rpc.api.Exchange_List_20260801_135631_235_RESP_0.bin");

pub const EXCHANGE_LIST_GACHA_COMMON_001_RESP: &[u8] =
    sniff!("rpc.api.Exchange_List_20260801_135624_780_RESP_0.bin");

pub const EXCHANGE_LIST_GACHA_BEGINNER_SELECT_001_RESP: &[u8] =
    sniff!("rpc.api.Exchange_List_20260801_135626_007_RESP_0.bin");

pub const EXCHANGE_LIST_MEMBERSHIP_CHR_00001_RESP: &[u8] =
    sniff!("rpc.api.Exchange_List_20260801_140731_300_RESP_0.bin");

#[cfg(test)]
mod tests {
    use prost::Message;
    use types::rpc::api::{
        AuthLoginResponse, EventListEventInfoResponse, ExchangeListResponse, GachaListResponse,
        HomeLoginResponse, LoginBonusCheckResponse, MasterGetResponse,
        MultiGameListPingServerResponse, NotificationListResponse, ParkEnterResponse,
        ParkRefreshResponse, StartupNotificationReadMusicResponse, StartupNotificationReadResponse,
        SystemGetSystemInfoResponse, UserGetResponse,
    };

    use super::*;

    #[test]
    fn all_sniff_bins_decode_into_their_types() {
        assert!(SystemGetSystemInfoResponse::decode(SYSTEM_GET_SYSTEM_INFO_RESP).is_ok());
        assert!(MasterGetResponse::decode(MASTER_GET_RESP).is_ok());
        assert!(AuthLoginResponse::decode(AUTH_LOGIN_RESP).is_ok());
        assert!(UserGetResponse::decode(USER_GET_RESP).is_ok());
        assert!(HomeLoginResponse::decode(HOME_LOGIN_RESP).is_ok());
        assert!(MultiGameListPingServerResponse::decode(MULTI_GAME_LIST_PING_SERVER_RESP).is_ok());
        assert!(ParkEnterResponse::decode(PARK_ENTER_RESP).is_ok());
        assert!(EventListEventInfoResponse::decode(EVENT_LIST_EVENT_INFO_RESP).is_ok());
        assert!(ParkRefreshResponse::decode(PARK_REFRESH_RESP).is_ok());
        assert!(NotificationListResponse::decode(NOTIFICATION_LIST_RESP).is_ok());
        assert!(StartupNotificationReadResponse::decode(STARTUP_NOTIFICATION_READ_RESP).is_ok());
        assert!(
            StartupNotificationReadMusicResponse::decode(STARTUP_NOTIFICATION_READ_MUSIC_RESP)
                .is_ok()
        );
        assert!(LoginBonusCheckResponse::decode(LOGIN_BONUS_CHECK_RESP).is_ok());
        assert!(GachaListResponse::decode(GACHA_LIST_RESP).is_ok());
        assert!(ExchangeListResponse::decode(EXCHANGE_LIST_GACHA_PICKUP_260728_RESP).is_ok());
        assert!(ExchangeListResponse::decode(EXCHANGE_LIST_GACHA_COMMON_001_RESP).is_ok());
        assert!(ExchangeListResponse::decode(EXCHANGE_LIST_GACHA_BEGINNER_SELECT_001_RESP).is_ok());
        assert!(ExchangeListResponse::decode(EXCHANGE_LIST_MEMBERSHIP_CHR_00001_RESP).is_ok());
    }
}
