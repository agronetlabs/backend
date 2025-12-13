use axum::{
    Json,
    extract::{Extension, State},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db;

#[derive(Clone)]
pub struct SettlementState {
    pub client: Arc<Client>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRequest {
    pub token_id: String,
    pub stablecoin: String,
    pub amount: f64,
    pub wallet_from: String,
    pub wallet_to: String,
}

pub async fn settle_stablecoin(
    State(_pool): State<db::DbPool>,
    Extension(_state): Extension<SettlementState>,
    Json(req): Json<SettlementRequest>,
) -> Json<serde_json::Value> {
    // mock de liquidação interno, suficiente para o fluxo do pull_liquidity
    let audit_hash = format!("ATF-AUDIT-{}", req.token_id);

    Json(serde_json::json!({
        "status": "settled",
        "token_id": req.token_id,
        "stablecoin": req.stablecoin,
        "amount": req.amount,
        "wallet_from": req.wallet_from,
        "wallet_to": req.wallet_to,
        "audit_hash": audit_hash
    }))
}
