CREATE TABLE onchain_settlement_log (
    id BIGSERIAL PRIMARY KEY,
    network TEXT NOT NULL,
    asset TEXT NOT NULL,
    amount NUMERIC NOT NULL,
    destination TEXT NOT NULL,
    audit_hash TEXT NOT NULL,
    token_id TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);
