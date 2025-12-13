use crate::db::DbPool;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

pub struct ValidationResult {
    pub approved: bool,
    pub reason: Option<String>,
}

pub async fn validate_onchain_request(
    pool: &DbPool,
    channel: &str,
    amount: Decimal,
    currency: &str,
) -> ValidationResult {
    let rule = sqlx::query!(
        r#"
        SELECT id, min_amount, max_amount, daily_limit
        FROM ledger_rules
        WHERE channel = $1 AND currency = $2 AND is_active = true
        LIMIT 1
        "#,
        channel,
        currency
    )
    .fetch_optional(pool)
    .await;

    let rule = match rule {
        Ok(Some(r)) => r,
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
    let daily_sum = sqlx::query_scalar!(
        r#"
        SELECT COALESCE(SUM(amount), 0) as "sum!: Decimal"
        FROM onchain_settlement_log
        WHERE network = $1
          AND asset = $2
          AND validation_status = 'approved'
          AND created_at >= $3::date
        "#,
        channel,
        currency,
        today_start
    )
    .fetch_one(pool)
    .await
    .unwrap_or_else(|_| Decimal::ZERO);

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
