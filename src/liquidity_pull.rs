use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

use crate::db::DbPool;
use crate::settlement::SettlementRequest; // reusa o tipo do seu settlement module

#[derive(Clone)]
pub struct LiquidityState {
    pub client: Arc<Client>,
    pub settlement_endpoint: String,
    pub file_path: PathBuf,
    pub max_pull_ratio: f64,
    pub safe_pull_limit: f64,
}

#[derive(Deserialize)]
struct StableFile {
    financial_info: Option<FinancialInfo>,
    crypto_data: Option<CryptoData>,
    // mantemos o resto ignorado por segurança
}

#[derive(Deserialize)]
struct FinancialInfo {
    total_balance: Option<String>, // pode vir como string
    currency: Option<String>,
    // outros campos omitidos
}

#[derive(Deserialize)]
struct CryptoData {
    contract_address: Option<String>,
    currency: Option<String>,
    chain: Option<String>,
}

/// cria LiquidityState a partir de env vars (safe defaults)
impl LiquidityState {
    pub fn from_env(client: Arc<Client>) -> Self {
        let file = std::env::var("LIQUIDITY_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/mnt/data/stablecoin-file.txt"));

        let endpoint = std::env::var("SETTLEMENT_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:8080/api/settlement/stable".to_string());

        let max_pull_ratio = std::env::var("MAX_PULL_RATIO")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.7);

        let safe_pull_limit = std::env::var("SAFE_PULL_LIMIT")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1_000_000.0);

        LiquidityState {
            client: client,
            settlement_endpoint: endpoint,
            file_path: file,
            max_pull_ratio,
            safe_pull_limit,
        }
    }
}

/// parseia `total_balance` que pode vir em formato string com separadores, devolve Option<f64>
fn parse_total_balance(s: &str) -> Option<f64> {
    // remove anything que nao seja digito, ponto ou vírgula
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_digit(10) || *c == '.' || *c == ',')
        .collect();

    // troca vírgula por ponto se houver
    let normalized = cleaned.replace(',', ".");

    normalized.parse::<f64>().ok()
}

fn validate_contract_address(addr_opt: &Option<String>) -> bool {
    if let Some(addr) = addr_opt {
        let re = Regex::new(r"^0x[0-9a-fA-F]{40}$").unwrap();
        re.is_match(addr)
    } else {
        false
    }
}

/// Função auxiliar: tenta detectar arquivo SWIFT/GPI + Blockchain
fn parse_swift_like(json: &serde_json::Value) -> Option<(f64, String, String)> {
    let swift = json.get("swift_transaction_details")?;
    let chain = json.get("blockchain_transaction")?;

    // saldo (CurrentBalance vem como string com vírgula)
    let balance_str = swift.get("CurrentBalance")?.as_str()?;
    let balance = balance_str.replace(",", "").parse::<f64>().ok()?;

    // stablecoin (ex: USDT)
    let stablecoin = swift.get("output_currency")?.as_str()?.to_string();

    // contract address (campo "to" do bloco)
    let contract = chain.get("to")?.as_str()?.to_string();

    Some((balance, stablecoin, contract))
}

use std::path::Path;

/// Lê arquivos de liquidez (.txt ou .fin) com validação de extensão
fn read_liquidity_file(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("file not found: {}", path.display()));
    }

    // aceita .txt, .fin e .json (modo ATF-AI Rv-offline)
    let allowed = match path.extension().and_then(|e| e.to_str()) {
        Some("txt") | Some("fin") | Some("json") => true,
        _ => false,
    };

    if !allowed {
        return Err(format!("unsupported file type: {}", path.display()));
    }

    fs::read_to_string(path).map_err(|e| format!("read_error: {}", e))
}

pub async fn liquidity_pull(
    State(pool): State<DbPool>,
    Extension(state): Extension<LiquidityState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // 1) ler arquivo protegido
    let content = match read_liquidity_file(&state.file_path) {
        Ok(s) => s,
        Err(e) => {
            return Ok(Json(serde_json::json!({
                "status": "error",
                "error": "read_error",
                "detail": format!("{}", e)
            })));
        }
    };

    // 2) carregar JSON genérico
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Ok(Json(serde_json::json!({
                "status": "error",
                "error": "parse_error",
                "detail": format!("{}", e),
            })));
        }
    };

    // 2A — TENTAR FORMATO SWIFT/GPI
    if let Some((balance, stablecoin, contract)) = parse_swift_like(&json) {
        return process_liquidity(balance, stablecoin, contract, state, &pool).await;
    }

    // 2B — Caso contrário, tentar formato antigo ERC-8040
    let parsed: StableFile = match serde_json::from_value(json.clone()) {
        Ok(p) => p,
        Err(_) => {
            return Ok(Json(serde_json::json!({
                "status": "error",
                "error": "invalid_format"
            })));
        }
    };

    // 3) extrair total_balance e currency (sem expor)
    let total_balance_opt = parsed
        .financial_info
        .as_ref()
        .and_then(|f| f.total_balance.as_ref())
        .and_then(|s| parse_total_balance(s));

    if total_balance_opt.is_none() {
        return Ok(Json(serde_json::json!({
            "status":"error",
            "error":"no_total_balance"
        })));
    }

    let total_balance = total_balance_opt.unwrap();

    // 4) validar contract address
    let contract_ok = validate_contract_address(
        &parsed
            .crypto_data
            .as_ref()
            .and_then(|c| c.contract_address.clone()),
    );

    if !contract_ok {
        return Ok(Json(serde_json::json!({
            "status":"error",
            "error":"invalid_contract_address"
        })));
    }

    // 5) calcular montante autorizavel + aplicar limite
    let mut authorized_amount = total_balance * state.max_pull_ratio;

    if authorized_amount > state.safe_pull_limit {
        authorized_amount = state.safe_pull_limit;
    }

    if !(authorized_amount > 0.0) || !authorized_amount.is_finite() {
        return Ok(Json(serde_json::json!({
            "status":"error",
            "error":"invalid_authorized_amount"
        })));
    }

    // 6) montar SettlementRequest
    let token_id = format!("ERC8040-{}", Utc::now().timestamp());

    let stablecoin = parsed
        .crypto_data
        .as_ref()
        .and_then(|c| c.currency.clone())
        .unwrap_or_else(|| "USDT".to_string());

    let req_body = SettlementRequest {
        token_id: token_id.clone(),
        stablecoin: stablecoin.clone(),
        amount: authorized_amount,
        wallet_from: "PRIVATE_SOURCE".to_string(),
        wallet_to: "TREASURY_DEST".to_string(),
    };

    // 7) chamar endpoint interno
    let resp = state
        .client
        .post(&state.settlement_endpoint)
        .json(&req_body)
        .send()
        .await;

    match resp {
        Ok(r) => match r.json::<serde_json::Value>().await {
            Ok(v) => {
                let audit_hash_opt = v
                    .get("audit_hash")
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string());

                if let Some(audit_hash) = audit_hash_opt.clone() {
                    sqlx::query(
                        "INSERT INTO settlement_liquidity (audit_hash, pulled_amount, stablecoin, token_id)
                         VALUES ($1, $2, $3, $4)",
                    )
                    .bind(audit_hash)
                    .bind(authorized_amount)
                    .bind(stablecoin.clone())
                    .bind(token_id.clone())
                    .execute(&pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }

                return Ok(Json(serde_json::json!({
                    "status":"ok",
                    "token_id": token_id,
                    "stablecoin": stablecoin,
                    "pulled_amount": authorized_amount,
                    "audit_hash": audit_hash_opt
                })));
            }
            Err(_) => {
                return Ok(Json(serde_json::json!({
                    "status":"ok",
                    "token_id": token_id,
                    "stablecoin": stablecoin,
                    "pulled_amount": authorized_amount,
                    "result":"unreadable_response"
                })));
            }
        },
        Err(e) => {
            return Ok(Json(serde_json::json!({
                "status":"error",
                "error":"request_failed",
                "detail": format!("{}", e)
            })));
        }
    }
}

async fn process_liquidity(
    balance: f64,
    stablecoin: String,
    contract: String,
    state: LiquidityState,
    pool: &DbPool,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // validar address
    let contract_ok = validate_contract_address(&Some(contract.clone()));
    if !contract_ok {
        return Ok(Json(
            serde_json::json!({"status":"error","error":"invalid_contract_address"}),
        ));
    }

    // calcular pull
    let mut authorized_amount = balance * state.max_pull_ratio;
    if authorized_amount > state.safe_pull_limit {
        authorized_amount = state.safe_pull_limit;
    }

    if !(authorized_amount > 0.0) || !authorized_amount.is_finite() {
        return Ok(Json(
            serde_json::json!({"status":"error","error":"invalid_authorized_amount"}),
        ));
    }

    // token ID
    let token_id = format!("ERC8040-{}", Utc::now().timestamp());

    // request
    let req_body = SettlementRequest {
        token_id: token_id.clone(),
        stablecoin: stablecoin.clone(),
        amount: authorized_amount,
        wallet_from: "PRIVATE_SOURCE".to_string(),
        wallet_to: "TREASURY_DEST".to_string(),
    };

    // chamada interna
    let resp = state
        .client
        .post(&state.settlement_endpoint)
        .json(&req_body)
        .send()
        .await;

    match resp {
        Ok(r) => match r.json::<serde_json::Value>().await {
            Ok(v) => {
                let audit_hash_opt = v
                    .get("audit_hash")
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string());

                if let Some(audit_hash) = audit_hash_opt.clone() {
                    sqlx::query(
                        "INSERT INTO settlement_liquidity (audit_hash, pulled_amount, stablecoin, token_id)
                         VALUES ($1, $2, $3, $4)",
                    )
                    .bind(audit_hash)
                    .bind(authorized_amount)
                    .bind(stablecoin.clone())
                    .bind(token_id.clone())
                    .execute(pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                }

                Ok(Json(serde_json::json!({
                    "status": "ok",
                    "token_id": token_id,
                    "stablecoin": stablecoin,
                    "pulled_amount": authorized_amount,
                    "audit_hash": audit_hash_opt
                })))
            }
            Err(_) => Ok(Json(serde_json::json!({
                "status":"ok",
                "token_id": token_id,
                "stablecoin": stablecoin,
                "pulled_amount": authorized_amount,
                "audit_hash": null
            }))),
        },
        Err(e) => Ok(Json(serde_json::json!({
            "status":"error",
            "error":"request_failed",
            "detail": e.to_string()
        }))),
    }
}
