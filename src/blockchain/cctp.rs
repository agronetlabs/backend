use crate::db::DbPool;
use crate::ledger::validate_onchain_request;
use axum::http::StatusCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize)]
pub struct OnChainResult {
    pub tx_hash: String,
    pub network: String,
    pub status: String,
    pub validation_status: String,
    pub validation_reason: Option<String>,
}

pub async fn send_liquidity_transaction(
    amount: Decimal,
    asset: &str,
    destination: &str,
    audit_hash: &str,
    token_id: &str,
    pool: DbPool,
) -> Result<OnChainResult, (StatusCode, axum::Json<Value>)> {
    let network = "cctp";

    let validation = validate_onchain_request(&pool, network, amount, asset).await;

    if !validation.approved {
        let reason_text = validation
            .reason
            .clone()
            .unwrap_or_else(|| "rejected".to_string());

        let _ = sqlx::query(
            "INSERT INTO onchain_settlement_log (network, asset, amount, destination, audit_hash, token_id, tx_hash, status, validation_status, validation_reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(network)
        .bind(asset)
        .bind(amount)
        .bind(destination)
        .bind(audit_hash)
        .bind(token_id)
        .bind("REJECTED")
        .bind("rejected")
        .bind("rejected")
        .bind(&reason_text)
        .execute(&pool)
        .await;

        return Err((
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "status": "rejected",
                "reason": reason_text,
                "channel": network,
                "amount": amount,
                "currency": asset
            })),
        ));
    }

    let validation_reason = "CCTP broadcaster not configured".to_string();
    let _ = sqlx::query(
        "INSERT INTO onchain_settlement_log (network, asset, amount, destination, audit_hash, token_id, tx_hash, status, validation_status, validation_reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(network)
    .bind(asset)
    .bind(amount)
    .bind(destination)
    .bind(audit_hash)
    .bind(token_id)
    .bind("NOT_CONFIGURED")
    .bind("not_configured")
    .bind("approved")
    .bind(&validation_reason)
    .execute(&pool)
    .await;

    Err((
        StatusCode::NOT_IMPLEMENTED,
        axum::Json(json!({
            "status": "error",
            "error": "cctp_not_configured",
            "detail": validation_reason,
            "channel": network
        })),
    ))
}
