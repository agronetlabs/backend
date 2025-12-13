CREATE TABLE ledger_rules (
    id BIGSERIAL PRIMARY KEY,
    channel TEXT NOT NULL,
    currency TEXT NOT NULL,
    min_amount NUMERIC NOT NULL,
    max_amount NUMERIC NOT NULL,
    daily_limit NUMERIC NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE onchain_settlement_log
    ADD COLUMN validation_status TEXT NOT NULL DEFAULT 'pending',
    ADD COLUMN validation_reason TEXT;
