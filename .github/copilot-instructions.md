# Copilot Custom Instructions for AgroNet Backend

## Project Context
This is a Rust backend for the AgroNet Labs ERC-8040 ecosystem - a financial settlement engine that bridges traditional finance (SWIFT) with blockchain (Ethereum, TRON, CCTP).

## Architecture
- **Axum** for REST API
- **SQLx** for PostgreSQL
- **tokio** for async runtime
- **reqwest** for HTTP client
- **rust_decimal** for financial precision

## Code Review Guidelines

### Security (CRITICAL)
- Always validate contract addresses with regex: `^0x[0-9a-fA-F]{40}$`
- Never expose private keys or wallet addresses in responses
- Validate all financial amounts are positive and finite
- Check for SQL injection in raw queries

### Financial Logic
- Use `rust_decimal::Decimal` for monetary values, not `f64` when precision matters
- Always apply `max_pull_ratio` and `safe_pull_limit` constraints
- Validate `daily_limit` before settlement

### ESG Compliance
- All settlements must include ESG scores when available
- SFDR and EU Taxonomy compliance is required for EU operations
- Environmental, Social, Governance scores range 0-100

### Blockchain
- Support multi-chain: Ethereum, TRON, CCTP
- All transactions must generate `audit_hash`
- Log all settlements to `onchain_settlement_log`

### Error Handling
- Return structured JSON errors with `status`, `error`, `detail`
- Never panic in production code
- Use `Result<T, E>` appropriately

### Testing
- Integration tests for all API endpoints
- Unit tests for validation functions
- Mock external services in tests

## File Structure
- `src/main.rs` - API routes and server
- `src/liquidity_pull.rs` - SWIFT JSON parser + liquidity engine
- `src/settlement.rs` - Settlement execution
- `src/ledger.rs` - Ledger rules validation
- `src/blockchain/*.rs` - Chain-specific settlement
- `src/esg/*.rs` - ESG validation module
- `src/db.rs` - Database connection

## Review Focus
1. Security vulnerabilities
2. Financial calculation accuracy
3. Proper error handling
4. ESG compliance integration
5. Audit trail completeness
