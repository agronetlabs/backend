-- ============================================================
-- LIQUIDITY BRIDGE: Exchange tables + credit function
-- Creates the minimal exchange infrastructure in agronet_db
-- so the backend can credit pulled liquidity into the exchange.
-- ============================================================

-- Users table (minimal, for treasury user)
CREATE TABLE IF NOT EXISTS users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  auth_id uuid,
  email text NOT NULL UNIQUE,
  display_name text NOT NULL DEFAULT '',
  role text NOT NULL DEFAULT 'trader' CHECK (role IN ('admin', 'trader')),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'pending')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Accounts table (balance tracking)
CREATE TABLE IF NOT EXISTS accounts (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id),
  asset text NOT NULL,
  available_balance numeric NOT NULL DEFAULT 0,
  reserved_balance numeric NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(user_id, asset)
);

-- Journals table (double-entry bookkeeping)
CREATE TABLE IF NOT EXISTS journals (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  reference_type text NOT NULL,
  reference_id uuid,
  description text NOT NULL DEFAULT '',
  status text NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'finalized')),
  finalized_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);

-- Ledger entries (append-only, double-entry)
CREATE TABLE IF NOT EXISTS ledger_entries (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  journal_id uuid NOT NULL REFERENCES journals(id),
  account_id uuid NOT NULL REFERENCES accounts(id),
  entry_type text NOT NULL CHECK (entry_type IN ('debit', 'credit')),
  amount numeric NOT NULL CHECK (amount > 0),
  description text NOT NULL DEFAULT '',
  created_at timestamptz NOT NULL DEFAULT now()
);

-- Events table (event sourcing)
CREATE TABLE IF NOT EXISTS events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  event_type text NOT NULL,
  aggregate_type text NOT NULL,
  aggregate_id uuid NOT NULL,
  payload jsonb NOT NULL DEFAULT '{}',
  stream text NOT NULL,
  stream_id uuid NOT NULL,
  seq bigint NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(stream, stream_id, seq)
);

-- Outbox (reliable event delivery)
CREATE TABLE IF NOT EXISTS outbox (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id uuid NOT NULL REFERENCES events(id),
  event_type text NOT NULL,
  aggregate_type text NOT NULL,
  aggregate_id uuid NOT NULL,
  payload jsonb NOT NULL DEFAULT '{}',
  status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'delivered', 'failed', 'dead')),
  locked_at timestamptz,
  locked_by text,
  attempts integer NOT NULL DEFAULT 0,
  max_attempts integer NOT NULL DEFAULT 5,
  next_attempt_at timestamptz NOT NULL DEFAULT now(),
  created_at timestamptz NOT NULL DEFAULT now(),
  delivered_at timestamptz
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_accounts_user_asset ON accounts(user_id, asset);
CREATE INDEX IF NOT EXISTS idx_ledger_entries_journal ON ledger_entries(journal_id);
CREATE INDEX IF NOT EXISTS idx_events_stream ON events(stream, stream_id, seq);

-- ============================================================
-- SEED: Treasury user + Liquidity Source user
-- ============================================================
INSERT INTO users (email, display_name, role, status)
VALUES
  ('system@treasury.internal', 'Treasury', 'admin', 'active'),
  ('system@liquidity-source.internal', 'Liquidity Source (External Fund)', 'admin', 'active')
ON CONFLICT (email) DO NOTHING;

-- ============================================================
-- FUNCTION: fn_get_treasury_account
-- ============================================================
CREATE OR REPLACE FUNCTION fn_get_treasury_account(p_asset text)
RETURNS uuid
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = 'public'
AS $$
DECLARE
  v_treasury_user_id uuid;
  v_account_id uuid;
BEGIN
  SELECT id INTO v_treasury_user_id FROM users WHERE email = 'system@treasury.internal';
  IF NOT FOUND THEN
    RAISE EXCEPTION 'FATAL: Treasury user system@treasury.internal not found';
  END IF;

  INSERT INTO accounts (user_id, asset, available_balance, reserved_balance)
  VALUES (v_treasury_user_id, p_asset, 0, 0)
  ON CONFLICT (user_id, asset) DO NOTHING;

  SELECT id INTO v_account_id
  FROM accounts WHERE user_id = v_treasury_user_id AND asset = p_asset;

  RETURN v_account_id;
END;
$$;

-- ============================================================
-- FUNCTION: fn_record_event
-- ============================================================
CREATE OR REPLACE FUNCTION fn_record_event(
  p_event_type text,
  p_aggregate_type text,
  p_aggregate_id uuid,
  p_payload jsonb DEFAULT '{}'::jsonb
) RETURNS uuid
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = 'public'
AS $$
DECLARE
  v_id uuid := gen_random_uuid();
  v_seq bigint;
BEGIN
  PERFORM 1 FROM events
  WHERE stream = p_aggregate_type AND stream_id = p_aggregate_id
  FOR UPDATE;

  SELECT COALESCE(MAX(seq), 0) + 1 INTO v_seq
  FROM events
  WHERE stream = p_aggregate_type AND stream_id = p_aggregate_id;

  INSERT INTO events (id, event_type, aggregate_type, aggregate_id, payload, stream, stream_id, seq)
  VALUES (v_id, p_event_type, p_aggregate_type, p_aggregate_id, p_payload, p_aggregate_type, p_aggregate_id, v_seq);

  INSERT INTO outbox (event_id, event_type, aggregate_type, aggregate_id, payload)
  VALUES (v_id, p_event_type, p_aggregate_type, p_aggregate_id, p_payload);

  RETURN v_id;
END;
$$;

-- ============================================================
-- FUNCTION: fn_credit_liquidity_pull
-- Called by the backend after a successful liquidity pull.
-- Credits the exchange treasury with the pulled amount.
-- ============================================================
CREATE OR REPLACE FUNCTION fn_credit_liquidity_pull(
  p_stablecoin text,
  p_amount numeric,
  p_audit_hash text,
  p_token_id text DEFAULT ''
) RETURNS jsonb
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = 'public'
AS $$
DECLARE
  v_treasury_acct uuid;
  v_source_user_id uuid;
  v_source_acct uuid;
  v_journal_id uuid;
  v_new_balance numeric;
BEGIN
  -- Validate
  IF p_amount <= 0 THEN
    RETURN jsonb_build_object('error', 'Amount must be positive');
  END IF;

  -- Get treasury account (exchange receives funds)
  v_treasury_acct := fn_get_treasury_account(p_stablecoin);

  -- Get/create liquidity source account (external fund debited)
  SELECT id INTO v_source_user_id FROM users WHERE email = 'system@liquidity-source.internal';
  IF NOT FOUND THEN
    RAISE EXCEPTION 'FATAL: Liquidity source user not found';
  END IF;

  INSERT INTO accounts (user_id, asset, available_balance, reserved_balance)
  VALUES (v_source_user_id, p_stablecoin, 0, 0)
  ON CONFLICT (user_id, asset) DO NOTHING;

  SELECT id INTO v_source_acct
  FROM accounts WHERE user_id = v_source_user_id AND asset = p_stablecoin;

  -- Credit treasury balance
  UPDATE accounts SET
    available_balance = available_balance + p_amount,
    updated_at = now()
  WHERE id = v_treasury_acct;

  SELECT available_balance INTO v_new_balance FROM accounts WHERE id = v_treasury_acct;

  -- Journal (double-entry: debit source, credit treasury)
  v_journal_id := gen_random_uuid();
  INSERT INTO journals (id, reference_type, reference_id, description, status, finalized_at)
  VALUES (
    v_journal_id, 'liquidity_pull', v_treasury_acct,
    format('Liquidity pull: %s %s | Audit: %s', p_amount, p_stablecoin, p_audit_hash),
    'finalized', now()
  );

  INSERT INTO ledger_entries (journal_id, account_id, entry_type, amount, description)
  VALUES
    (v_journal_id, v_source_acct, 'debit', p_amount,
      format('External fund debit for liquidity pull %s %s', p_amount, p_stablecoin)),
    (v_journal_id, v_treasury_acct, 'credit', p_amount,
      format('Treasury credit from liquidity pull %s %s', p_amount, p_stablecoin));

  -- Emit event
  PERFORM fn_record_event(
    'LiquidityPullCredited',
    'account',
    v_treasury_acct,
    jsonb_build_object(
      'stablecoin', p_stablecoin,
      'amount', p_amount,
      'audit_hash', p_audit_hash,
      'token_id', p_token_id,
      'journal_id', v_journal_id,
      'treasury_balance', v_new_balance
    )
  );

  RETURN jsonb_build_object(
    'treasury_account_id', v_treasury_acct,
    'treasury_balance', v_new_balance,
    'journal_id', v_journal_id,
    'audit_hash', p_audit_hash
  );
END;
$$;
