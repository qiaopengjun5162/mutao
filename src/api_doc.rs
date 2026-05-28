use utoipa::OpenApi;

use crate::auth::{LoginReq, LoginRes};
use crate::handlers::auth_handler::RegisterReq;
use crate::handlers::demands::CreateDemandReq;
use crate::handlers::items::{CreateItemReq, UpdateStatusReq};
use crate::models::{Demand, Item, ItemStatus, SwapCycle, SwapLeg, User};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "木桃 Mutao API",
        description = "AI 撮合 + Web3 溯源的免现金实体易物平台",
        version = "0.1.0"
    ),
    components(schemas(
        Item,
        ItemStatus,
        Demand,
        SwapCycle,
        SwapLeg,
        User,
        CreateItemReq,
        UpdateStatusReq,
        CreateDemandReq,
        RegisterReq,
        LoginReq,
        LoginRes,
    ))
)]
pub struct ApiDoc;
