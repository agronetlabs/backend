use crate::db::DbPool;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::Row;

pub struct ValidationResult {
    pub approved: bool,
    pub reason: Option<String>,
}

#[derive(Debug)]
struct LedgerRule {
    #[allow(dead_code)]
    id: i64,
    min_amount: Decimal,
    max_amount: Decimal,
    daily_limit: Decimal,
}

pub async fn validate_onchain_request(
    pool: &DbPool,
    channel: &str,
    amount: Decimal,
    currency: &str,
) -> ValidationResult {
    // Avoid SQLx `query!` macros so builds/tests don't require a live DB connection
    // at compile time.
    let rule = sqlx::query(
        r#"
        SELECT id, min_amount, max_amount, daily_limit
        FROM ledger_rules
        WHERE channel = $1 AND currency = $2 AND is_active = true
        LIMIT 1
        "#,
    )
    .bind(channel)
    .bind(currency)
    .fetch_optional(pool)
    .await;

    let rule = match rule {
        Ok(Some(row)) => {
            let id: i64 = match row.try_get("id") {
                Ok(v) => v,
                Err(e) => {
                    return ValidationResult {
                        approved: false,
                        reason: Some(format!("Validation decode failed: {e}")),
                    };
                }
            };
            let min_amount: Decimal = match row.try_get("min_amount") {
                Ok(v) => v,
                Err(e) => {
                    return ValidationResult {
                        approved: false,
                        reason: Some(format!("Validation decode failed: {e}")),
                    };
                }
            };
            let max_amount: Decimal = match row.try_get("max_amount") {
                Ok(v) => v,
                Err(e) => {
                    return ValidationResult {
                        approved: false,
                        reason: Some(format!("Validation decode failed: {e}")),
                    };
                }
            };
            let daily_limit: Decimal = match row.try_get("daily_limit") {
                Ok(v) => v,
                Err(e) => {
                    return ValidationResult {
                        approved: false,
                        reason: Some(format!("Validation decode failed: {e}")),
                    };
                }
            };

            LedgerRule {
                id,
                min_amount,
                max_amount,
                daily_limit,
            }
        }
        Ok(None) => {
            return ValidationResult {
                approved: false,
                reason: Some("No active ledger rule for channel/currency".to_string()),
            };
        }
        Err(e) => {
            return ValidationResult {
                approved: false,
                reason: Some(format!("Validation query failed: {}", e)),
            };
        }
    };

    if amount < rule.min_amount || amount > rule.max_amount {
        return ValidationResult {
            approved: false,
            reason: Some("Amount outside allowed range".to_string()),
        };
    }

    // Optional daily limit check
    let today_start: NaiveDate = Utc::now().date_naive();
    let daily_sum: Decimal = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount), 0)
        FROM onchain_settlement_log
        WHERE network = $1
          AND asset = $2
          AND validation_status = 'approved'
          AND created_at >= $3::date
        "#,
    )
    .bind(channel)
    .bind(currency)
    .bind(today_start)
    .fetch_one(pool)
    .await
    .unwrap_or(Decimal::ZERO);

    if daily_sum + amount > rule.daily_limit {
        return ValidationResult {
            approved: false,
            reason: Some("Daily limit exceeded".to_string()),
        };
    }

    ValidationResult {
        approved: true,
        reason: Some("OK".to_string()),
    }
}
