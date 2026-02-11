ALTER TABLE settlement_liquidity
  DROP COLUMN IF EXISTS source_format,
  DROP COLUMN IF EXISTS total_balance,
  DROP COLUMN IF EXISTS max_pull_ratio,
  DROP COLUMN IF EXISTS safe_pull_limit,
  DROP COLUMN IF EXISTS contract_address,
  DROP COLUMN IF EXISTS status;

ALTER TABLE ledger_rules
  DROP COLUMN IF EXISTS updated_at;
