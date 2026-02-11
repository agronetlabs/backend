-- Add columns expected by CEXS.io frontend to settlement_liquidity
ALTER TABLE settlement_liquidity
  ADD COLUMN IF NOT EXISTS source_format TEXT NOT NULL DEFAULT 'ERC-8040',
  ADD COLUMN IF NOT EXISTS total_balance NUMERIC NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS max_pull_ratio NUMERIC NOT NULL DEFAULT 0.70,
  ADD COLUMN IF NOT EXISTS safe_pull_limit NUMERIC NOT NULL DEFAULT 1000000,
  ADD COLUMN IF NOT EXISTS contract_address TEXT NOT NULL DEFAULT '',
  ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'completed';

-- Add updated_at to ledger_rules for frontend compatibility
ALTER TABLE ledger_rules
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
