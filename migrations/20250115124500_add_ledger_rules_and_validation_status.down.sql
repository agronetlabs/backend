ALTER TABLE onchain_settlement_log
    DROP COLUMN IF EXISTS validation_status,
    DROP COLUMN IF EXISTS validation_reason;

DROP TABLE IF EXISTS ledger_rules;
