use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementRequest {
    pub token_id: String,
    pub stablecoin: String,
    pub amount: f64,
    pub wallet_from: String,
    pub wallet_to: String,
}

fn validate_settlement_request(req: &SettlementRequest) -> Result<(), String> {
    if req.token_id.trim().is_empty() {
        return Err("token_id is required".to_string());
    }
    if req.stablecoin.trim().is_empty() {
        return Err("stablecoin is required".to_string());
    }
    if req.wallet_from.trim().is_empty() || req.wallet_to.trim().is_empty() {
        return Err("wallet_from and wallet_to are required".to_string());
    }
    if !req.amount.is_finite() || req.amount <= 0.0 {
        return Err("amount must be a positive finite number".to_string());
    }
    Ok(())
}

fn build_audit_hash(req: &SettlementRequest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(req.token_id.as_bytes());
    hasher.update(req.stablecoin.as_bytes());
    hasher.update(req.amount.to_string().as_bytes());
    hasher.update(req.wallet_from.as_bytes());
    hasher.update(req.wallet_to.as_bytes());
    let digest = hasher.finalize();
    let short = hex::encode(digest);
    format!("ATF-AI-AUDIT-{}", short[..32].to_uppercase())
}

pub fn execute_settlement(req: &SettlementRequest) -> Result<serde_json::Value, (StatusCode, String)> {
    validate_settlement_request(req).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let audit_hash = build_audit_hash(req);
    Ok(serde_json::json!({
        "status": "ok",
        "token_id": req.token_id,
        "stablecoin": req.stablecoin,
        "amount": req.amount,
        "audit_hash": audit_hash
    }))
}

pub async fn settle_stablecoin(
    State(_pool): State<db::DbPool>,
    Json(req): Json<SettlementRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match execute_settlement(&req) {
        Ok(body) => (StatusCode::OK, Json(body)),
        Err((status, detail)) => (
            status,
            Json(serde_json::json!({
                "status": "error",
                "error": "invalid_settlement_request",
                "detail": detail
            })),
        ),
    }
}
