CREATE TABLE settlement_liquidity (
    id BIGSERIAL PRIMARY KEY,
    audit_hash TEXT NOT NULL,
    pulled_amount NUMERIC NOT NULL,
    stablecoin TEXT NOT NULL,
    token_id TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);
