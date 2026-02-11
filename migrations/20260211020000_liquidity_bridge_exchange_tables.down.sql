DROP FUNCTION IF EXISTS fn_credit_liquidity_pull(text, numeric, text, text);
DROP FUNCTION IF EXISTS fn_record_event(text, text, uuid, jsonb);
DROP FUNCTION IF EXISTS fn_get_treasury_account(text);
DROP TABLE IF EXISTS outbox;
DROP TABLE IF EXISTS events;
DROP TABLE IF EXISTS ledger_entries;
DROP TABLE IF EXISTS journals;
DROP TABLE IF EXISTS accounts;
DROP TABLE IF EXISTS users;
