use database::Database;
use tonic::{Request, Response, Status};
use types::Validate;
use types::enums::NoticeDisplayType;
use types::rpc::api::notice_server::Notice;
use types::rpc::api::notice_top_response;
use types::rpc::api::{
    NoticeGetRequest, NoticeGetResponse, NoticeListInCategoryRequest, NoticeListInCategoryResponse,
    NoticeTopResponse, NoticeUpdateCategoryReadTimeRequest, NoticeUpdateCategoryReadTimeResponse,
    NoticeUpdateDetailReadTimeRequest, NoticeUpdateDetailReadTimeResponse,
};

mod notice_data;

use notice_data::notices;

#[derive(Clone)]
pub struct NoticeService {
    db: Database,
}

impl NoticeService {
    pub async fn init(db: Database) -> Result<Self, String> {
        Ok(Self { db })
    }

    /// the catalog with each notice's read state from the DB.
    async fn catalog(&self, uid: Option<&str>) -> Vec<types::rpc::api::NoticeInfo> {
        let mut list = notices();
        for n in &mut list {
            let read = match uid {
                Some(uid) => self
                    .db
                    .notice_read_time(uid, &n.id)
                    .await
                    .map(|t| t.is_some())
                    .unwrap_or(false),
                None => false,
            };
            n.is_read = read;
        }
        list
    }
}

#[tonic::async_trait]
impl Notice for NoticeService {
    async fn top(&self, _req: Request<()>) -> Result<Response<NoticeTopResponse>, Status> {
        let uid = crate::auth_token::uid_opt(&_req);
        let list = self.catalog(uid.as_deref()).await;
        Ok(Response::new(NoticeTopResponse {
            categories: vec![notice_top_response::Category {
                notice_category_id: "notice_category-news".into(),
                name: "List".into(),
                display_type: NoticeDisplayType::Normal as i32,
                notice_infos: list,
                is_has_next: false,
            }],
            common_response: None,
        }))
    }

    async fn get(
        &self,
        req: Request<NoticeGetRequest>,
    ) -> Result<Response<NoticeGetResponse>, Status> {
        let uid = crate::auth_token::uid_opt(&req);
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let list = self.catalog(uid.as_deref()).await;
        let Some(notice_info) = list.into_iter().find(|n| n.id == req.notice_id) else {
            return Err(Status::not_found("unknown notice_id"));
        };
        Ok(Response::new(NoticeGetResponse {
            notice_info: Some(notice_info),
            common_response: None,
        }))
    }

    async fn list_in_category(
        &self,
        req: Request<NoticeListInCategoryRequest>,
    ) -> Result<Response<NoticeListInCategoryResponse>, Status> {
        let uid = crate::auth_token::uid_opt(&req);
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let list = self.catalog(uid.as_deref()).await;
        let notice_infos = if req.notice_category_id == "notice_category-news" {
            list
        } else {
            vec![]
        };
        Ok(Response::new(NoticeListInCategoryResponse {
            notice_infos,
            is_has_next: false,
            common_response: None,
        }))
    }

    async fn update_detail_read_time(
        &self,
        req: Request<NoticeUpdateDetailReadTimeRequest>,
    ) -> Result<Response<NoticeUpdateDetailReadTimeResponse>, Status> {
        let uid = crate::auth_token::uid(&req)?;
        let req = req.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let now = database::unix_now() * 1000;
        for id in &req.notice_ids {
            self.db
                .set_notice_read_time(&uid, id, now)
                .await
                .map_err(|e| Status::internal(format!("notice read: {e}")))?;
        }
        Ok(Response::new(NoticeUpdateDetailReadTimeResponse::default()))
    }

    async fn update_category_read_time(
        &self,
        req: Request<NoticeUpdateCategoryReadTimeRequest>,
    ) -> Result<Response<NoticeUpdateCategoryReadTimeResponse>, Status> {
        req.into_inner()
            .validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        Ok(Response::new(
            NoticeUpdateCategoryReadTimeResponse::default(),
        ))
    }
}
