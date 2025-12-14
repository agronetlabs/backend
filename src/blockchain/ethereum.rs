use crate::db::DbPool;
use crate::ledger::validate_onchain_request;
use axum::http::StatusCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct OnChainResult {
    pub tx_hash: String,
    pub network: String,
    pub status: String,
    pub validation_status: String,
    pub validation_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_url: Option<String>,
}

pub async fn send_liquidity_transaction(
    amount: Decimal,
    asset: &str,
    destination: &str,
    audit_hash: &str,
    token_id: &str,
    pool: DbPool,
) -> Result<OnChainResult, (StatusCode, axum::Json<Value>)> {
    let network = "ethereum";

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

    let tx_hash = Uuid::new_v4().to_string();
    let status = "mocked";
    let validation_status = "approved";
    let validation_reason = validation.reason.unwrap_or_else(|| "OK".to_string());

    sqlx::query(
        "INSERT INTO onchain_settlement_log (network, asset, amount, destination, audit_hash, token_id, tx_hash, status, validation_status, validation_reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(network)
    .bind(asset)
    .bind(amount)
    .bind(destination)
    .bind(audit_hash)
    .bind(token_id)
    .bind(&tx_hash)
    .bind(status)
    .bind(validation_status)
    .bind(&validation_reason)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok(OnChainResult {
        tx_hash,
        network: network.to_string(),
        status: status.to_string(),
        validation_status: validation_status.to_string(),
        validation_reason: Some(validation_reason),
        block_number: None,
        gas_used: None,
        explorer_url: None,
    })
}

/// Send a real Ethereum transaction using ethers-rs
/// This function requires a signer to be configured via environment variables
#[allow(dead_code)]
pub async fn send_liquidity_transaction_real(
    amount: Decimal,
    asset: &str,
    destination: &str,
    audit_hash: &str,
    token_id: &str,
    pool: DbPool,
    network_name: Option<&str>,
) -> Result<OnChainResult, (StatusCode, axum::Json<Value>)> {
    use super::contracts::IERC20;
    use super::networks::Network;
    use super::provider::get_provider;
    use ethers::prelude::*;
    use std::str::FromStr;
    use std::sync::Arc;

    let network_str = network_name.unwrap_or("ethereum");
    let network = Network::from_str(network_str).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "invalid_network",
                "detail": e
            })),
        )
    })?;

    // Validate the request first
    let validation = validate_onchain_request(&pool, network_str, amount, asset).await;

    if !validation.approved {
        let reason_text = validation
            .reason
            .clone()
            .unwrap_or_else(|| "rejected".to_string());

        let _ = sqlx::query(
            "INSERT INTO onchain_settlement_log (network, asset, amount, destination, audit_hash, token_id, tx_hash, status, validation_status, validation_reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(network_str)
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
                "channel": network_str,
                "amount": amount,
                "currency": asset
            })),
        ));
    }

    // Get provider
    let provider = get_provider(network_str).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "provider_error",
                "detail": e.to_string()
            })),
        )
    })?;

    // Parse destination address
    let destination_addr = Address::from_str(destination).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "invalid_address",
                "detail": e.to_string()
            })),
        )
    })?;

    // Get token address based on asset
    let token_address = match asset.to_uppercase().as_str() {
        "USDT" => network.usdt_address(),
        "USDC" => network.usdc_address(),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                axum::Json(json!({
                    "error": "unsupported_asset",
                    "detail": format!("Asset {} not supported", asset)
                })),
            ));
        }
    };

    // Check if we have a private key configured
    let private_key = std::env::var("ETH_PRIVATE_KEY").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "configuration_error",
                "detail": "ETH_PRIVATE_KEY not configured"
            })),
        )
    })?;

    // Create wallet and signer
    let wallet: LocalWallet = private_key.parse::<LocalWallet>().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "wallet_error",
                "detail": e.to_string()
            })),
        )
    })?;

    let chain_id = network.chain_id();
    let wallet = wallet.with_chain_id(chain_id);

    // Create client with signer
    let client = SignerMiddleware::new(provider, wallet);
    let client = Arc::new(client);

    // Create contract instance
    let contract = IERC20::new(token_address, client.clone());

    // Get token decimals from contract
    let decimals = contract.decimals().call().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "decimals_fetch_failed",
                "detail": e.to_string()
            })),
        )
    })? as u32;

    // Convert amount to token's base unit with proper precision
    let multiplier = Decimal::from(10u64.pow(decimals));
    let amount_scaled = amount * multiplier;
    
    // Convert to U256 safely
    let amount_str = amount_scaled.to_string();
    let amount_parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part = amount_parts.get(0).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "invalid_amount",
                "detail": "Failed to parse amount"
            })),
        )
    })?;
    
    let amount_wei = U256::from_dec_str(integer_part).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(json!({
                "error": "invalid_amount",
                "detail": format!("Failed to convert amount: {}", e)
            })),
        )
    })?;

    // Execute transfer
    let tx = contract.transfer(destination_addr, amount_wei);
    
    let pending_tx = tx.send().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "transaction_failed",
                "detail": e.to_string()
            })),
        )
    })?;

    let tx_hash = format!("{:?}", pending_tx.tx_hash());

    // Wait for confirmation
    let receipt = pending_tx.await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "confirmation_failed",
                "detail": e.to_string()
            })),
        )
    })?;

    let status = if receipt.as_ref().and_then(|r| r.status).map(|s| s.as_u64()).unwrap_or(0) == 1 {
        "confirmed"
    } else {
        "failed"
    };

    let block_number = receipt.as_ref().and_then(|r| r.block_number).map(|b| b.as_u64());
    let gas_used = receipt.as_ref().and_then(|r| r.gas_used).map(|g| g.to_string());

    let explorer_url = format!("{}/tx/{}", network.explorer(), tx_hash);

    let validation_reason = validation.reason.unwrap_or_else(|| "OK".to_string());

    // Log to database
    sqlx::query(
        "INSERT INTO onchain_settlement_log (network, asset, amount, destination, audit_hash, token_id, tx_hash, status, validation_status, validation_reason)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(network_str)
    .bind(asset)
    .bind(amount)
    .bind(destination)
    .bind(audit_hash)
    .bind(token_id)
    .bind(&tx_hash)
    .bind(status)
    .bind("approved")
    .bind(&validation_reason)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"error": e.to_string()})),
        )
    })?;

    Ok(OnChainResult {
        tx_hash,
        network: network_str.to_string(),
        status: status.to_string(),
        validation_status: "approved".to_string(),
        validation_reason: Some(validation_reason),
        block_number,
        gas_used,
        explorer_url: Some(explorer_url),
    })
}
