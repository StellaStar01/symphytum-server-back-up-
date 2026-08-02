use tonic::{Request, Response, Status};

use types::rpc::api::exchange_server::Exchange;
use types::rpc::api::{
    ExchangeListRequest, ExchangeListResponse, ExchangePurchaseRequest, ExchangePurchaseResponse,
    ExchangeReadBoothRequest, ExchangeReadBoothResponse,
};

use crate::services::replay;
use crate::sniffs;

#[derive(Default)]
pub struct ExchangeService {}

#[tonic::async_trait]
impl Exchange for ExchangeService {
    async fn list(
        &self,
        req: Request<ExchangeListRequest>,
    ) -> Result<Response<ExchangeListResponse>, Status> {
        let group = req.into_inner().booth_group_id;
        let resp = match group.as_str() {
            "exchange_booth_group-gacha-pickup-260728" => replay!(
                ExchangeListResponse,
                sniffs::EXCHANGE_LIST_GACHA_PICKUP_260728_RESP
            ),
            "exchange_booth_group-gacha-common-001" => replay!(
                ExchangeListResponse,
                sniffs::EXCHANGE_LIST_GACHA_COMMON_001_RESP
            ),
            "exchange_booth_group-gacha-beginner-select-001" => replay!(
                ExchangeListResponse,
                sniffs::EXCHANGE_LIST_GACHA_BEGINNER_SELECT_001_RESP
            ),
            "exchange_booth_group-membership-chr-00001" => replay!(
                ExchangeListResponse,
                sniffs::EXCHANGE_LIST_MEMBERSHIP_CHR_00001_RESP
            ),
            // no capture for this group: a valid empty list keeps the menu working
            _ => ExchangeListResponse {
                id: group,
                ..Default::default()
            },
        };
        Ok(Response::new(resp))
    }

    async fn purchase(
        &self,
        _req: Request<ExchangePurchaseRequest>,
    ) -> Result<Response<ExchangePurchaseResponse>, Status> {
        Err(Status::unimplemented("Exchange.purchase"))
    }

    async fn read_booth(
        &self,
        _req: Request<ExchangeReadBoothRequest>,
    ) -> Result<Response<ExchangeReadBoothResponse>, Status> {
        Err(Status::unimplemented("Exchange.read_booth"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_replays_captured_groups() {
        let svc = ExchangeService::default();
        for id in [
            "exchange_booth_group-gacha-common-001",
            "exchange_booth_group-gacha-pickup-260728",
            "exchange_booth_group-gacha-beginner-select-001",
            "exchange_booth_group-membership-chr-00001",
        ] {
            let resp = svc
                .list(Request::new(ExchangeListRequest {
                    booth_group_id: id.into(),
                }))
                .await
                .expect("list must not error");
            let inner = resp.into_inner();
            assert_eq!(inner.id, id);
            assert!(!inner.booths.is_empty(), "{id}: no booths");
            assert!(
                inner.booths.iter().all(|b| !b.items.is_empty()),
                "{id}: a booth without items"
            );
        }
    }

    #[tokio::test]
    async fn list_unknown_group_returns_valid_empty() {
        let svc = ExchangeService::default();
        let resp = svc
            .list(Request::new(ExchangeListRequest {
                booth_group_id: "exchange_booth_group-nope".into(),
            }))
            .await
            .expect("unknown group must not error");
        let inner = resp.into_inner();
        assert_eq!(inner.id, "exchange_booth_group-nope");
        assert!(inner.booths.is_empty());
    }
}
