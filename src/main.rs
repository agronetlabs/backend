mod blockchain;
mod db;
mod ledger;
mod ledger_flex;
mod liquidity_pull;
use liquidity_pull::{LiquidityState, liquidity_pull};
mod settlement;
use crate::blockchain::{cctp, ethereum, tron};
use settlement::settle_stablecoin;

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sled::Db;
use std::{env, net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    db: Arc<Db>,
    jwt_secret: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: String,
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Deserialize)]
struct OnChainRequest {
    amount: Decimal,
    asset: Option<String>,
    currency: Option<String>,
    destination: String,
    audit_hash: Option<String>,
    token_id: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let sled_db = sled::open("data/db").expect("failed to open sled db");

    let pool = match db::connect().await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "failed to connect to PostgreSQL (check DATABASE_URL)");
            return;
        }
    };

    let app_state = Arc::new(AppState {
        db: Arc::new(sled_db),
        jwt_secret,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let liquidity_state = LiquidityState::from_env();

    let liquidity_router = Router::new()
        .route("/api/settlement/pull_liquidity", post(liquidity_pull))
        .layer(Extension(liquidity_state.clone()));

    let settlement_router = Router::new().route("/api/settlement/stable", post(settle_stablecoin));

    let app = Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(me))
        .route("/api/dashboard/summary", get(summary))
        .route("/api/ai/validate", post(ai_validate))
        .route("/api/ai/explain", post(ai_explain))
        .route("/api/onchain/settle/tron", post(settle_tron))
        .route("/api/onchain/settle/cctp", post(settle_cctp))
        .route("/api/onchain/settle/ethereum", post(settle_ethereum))
        .route("/api/exchange/treasury", get(get_treasury_balance))
        .layer(
            ServiceBuilder::new()
                .layer(cors)
                .layer(Extension(app_state.clone())),
        )
        .merge(liquidity_router)
        .merge(settlement_router);
    let app = app.with_state(pool);

    let bind_ip = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr: SocketAddr = format!("{bind_ip}:{port}")
        .parse()
        .expect("BIND_ADDR/PORT must form a valid socket address");
    tracing::info!("Server running on {}", addr);

    use axum::Server;

    let server = match Server::try_bind(&addr) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to bind server socket");
            return;
        }
    };

    server
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn register(
    State(_pool): State<db::DbPool>,
    Extension(state): Extension<Arc<AppState>>,
    Json(user): Json<User>,
) -> impl IntoResponse {
    state
        .db
        .insert(user.email.clone(), serde_json::to_vec(&user).unwrap())
        .unwrap();
    (StatusCode::OK, "User registered")
}

async fn login(
    State(_pool): State<db::DbPool>,
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    if let Ok(Some(data)) = state.db.get(&req.email) {
        let user: User = serde_json::from_slice(&data).unwrap();

        if user.password == req.password {
            let expiration = Utc::now()
                .checked_add_signed(Duration::hours(24))
                .unwrap()
                .timestamp() as usize;

            let claims = Claims {
                sub: req.email.clone(),
                exp: expiration,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
            )
            .unwrap();

            return (StatusCode::OK, Json(serde_json::json!({ "token": token })));
        }
    }

    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "error": "Invalid credentials" })),
    )
}

async fn me(
    State(_pool): State<db::DbPool>,
    Extension(state): Extension<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_header = headers.get("Authorization");

    if auth_header.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Missing token" })),
        );
    }

    let token = auth_header
        .unwrap()
        .to_str()
        .unwrap()
        .trim_start_matches("Bearer ")
        .to_string();

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    );

    match decoded {
        Ok(data) => (
            StatusCode::OK,
            Json(serde_json::json!({ "email": data.claims.sub })),
        ),
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid or expired token" })),
        ),
    }
}

async fn summary() -> impl IntoResponse {
    Json(serde_json::json!({
        "balance": 1200.50,
        "assets": 3,
        "transactions": 42
    }))
}

async fn ai_validate(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    let resp = client
        .post("http://127.0.0.1:8001/validate")
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    Json(resp)
}

async fn ai_explain(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    let resp = client
        .post("http://127.0.0.1:8001/explain")
        .json(&payload)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    Json(resp)
}

async fn get_treasury_balance(
    State(pool): State<db::DbPool>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (String, rust_decimal::Decimal)>(
        "SELECT a.asset, a.available_balance FROM accounts a
         JOIN users u ON u.id = a.user_id
         WHERE u.email = 'system@treasury.internal' AND a.available_balance > 0"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let balances: Vec<serde_json::Value> = rows
        .iter()
        .map(|(asset, balance)| json!({"asset": asset, "balance": balance}))
        .collect();

    Ok(Json(json!({ "status": "ok", "treasury": balances })))
}

async fn settle_tron(
    State(pool): State<db::DbPool>,
    Json(req): Json<OnChainRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let asset = req
        .asset
        .or(req.currency)
        .unwrap_or_else(|| "USDT".to_string());
    let audit_hash = req.audit_hash.unwrap_or_else(|| "N/A".to_string());
    let token_id = req.token_id.unwrap_or_else(|| "N/A".to_string());

    match tron::send_liquidity_transaction(
        req.amount,
        &asset,
        &req.destination,
        &audit_hash,
        &token_id,
        pool,
    )
    .await
    {
        Ok(res) => Ok(Json(json!(res))),
        Err((status, body)) => Err((status, body)),
    }
}

async fn settle_cctp(
    State(pool): State<db::DbPool>,
    Json(req): Json<OnChainRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let asset = req
        .asset
        .or(req.currency)
        .unwrap_or_else(|| "USDT".to_string());
    let audit_hash = req.audit_hash.unwrap_or_else(|| "N/A".to_string());
    let token_id = req.token_id.unwrap_or_else(|| "N/A".to_string());

    match cctp::send_liquidity_transaction(
        req.amount,
        &asset,
        &req.destination,
        &audit_hash,
        &token_id,
        pool,
    )
    .await
    {
        Ok(res) => Ok(Json(json!(res))),
        Err((status, body)) => Err((status, body)),
    }
}

async fn settle_ethereum(
    State(pool): State<db::DbPool>,
    Json(req): Json<OnChainRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    let asset = req
        .asset
        .or(req.currency)
        .unwrap_or_else(|| "USDT".to_string());
    let audit_hash = req.audit_hash.unwrap_or_else(|| "N/A".to_string());
    let token_id = req.token_id.unwrap_or_else(|| "N/A".to_string());

    match ethereum::send_liquidity_transaction(
        req.amount,
        &asset,
        &req.destination,
        &audit_hash,
        &token_id,
        pool,
    )
    .await
    {
        Ok(res) => Ok(Json(json!(res))),
        Err((status, body)) => Err((status, body)),
    }
}
