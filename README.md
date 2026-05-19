# AgroNet Backend — Settlement & Liquidity Infrastructure

> Institutional-grade settlement engine built in Rust. Part of the [ATF-AI](https://github.com/agronetlabs/ATF-AI) ecosystem.

![Build](https://img.shields.io/badge/Build-Passing-brightgreen?style=for-the-badge)
![Tests](https://img.shields.io/badge/Tests-10%2F10%20Passing-brightgreen?style=for-the-badge)
![Language](https://img.shields.io/badge/Rust-Axum-orange?style=for-the-badge&logo=rust)
![License](https://img.shields.io/badge/License-Proprietary-red?style=for-the-badge)

---

## What This Is

The AgroNet Backend is the settlement and liquidity engine powering **CEXS.io** — an institutional-grade digital asset exchange built natively on the ATF-AI Autonomous Trust Framework.

Every settlement operation automatically generates an **ATF-AI audit hash** — a cryptographic proof of provenance embedded in every transaction.

---

## ✅ Proof of Build

### Settlement Running Live

![Settlement Proof](assets/build-proof-settlement.jpg)

`POST /api/settlement/pull_liquidity` responding with USDT settlement, ATF-AI audit hash generated automatically.

### 10/10 Tests Passing — Server Running

![Build Proof](assets/build-proof-tests.jpg)

Clean Rust build, all unit tests passing, server live on port 8080.

---

## Architecture

```
CEXS.io PWA (Frontend)
        ↓
AgroNet Backend (This repo — Rust/Axum)
        ↓
┌───────────────────────────────────┐
│  Settlement Layer                 │
│  ├── TRON (USDT/USDC)            │
│  ├── Ethereum (ERC-20)            │
│  └── CCTP (Cross-Chain)          │
├───────────────────────────────────┤
│  ATF-AI Compliance Layer          │
│  └── Audit hash on every tx       │
├───────────────────────────────────┤
│  Liquidity Layer                  │
│  └── Pull liquidity from pools    │
└───────────────────────────────────┘
        ↓
PostgreSQL (Double-entry ledger)
```

---

## API Endpoints

### Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/auth/register` | Register user |
| POST | `/api/auth/login` | Login + JWT |
| GET | `/api/auth/me` | Current user |

### Settlement
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/onchain/settle/tron` | Settle on TRON network |
| POST | `/api/onchain/settle/ethereum` | Settle on Ethereum |
| POST | `/api/onchain/settle/cctp` | Cross-Chain Transfer Protocol |
| POST | `/api/settlement/stable` | Stablecoin settlement |
| POST | `/api/settlement/pull_liquidity` | Pull liquidity |

### Exchange
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/exchange/treasury` | Treasury balance |
| GET | `/api/dashboard/summary` | Dashboard summary |

### AI Compliance
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/ai/validate` | ATF-AI validation |
| POST | `/api/ai/explain` | ATF-AI explanation |

---

## ATF-AI Integration

Every settlement generates a cryptographic audit hash:

```
ATF-AI-AUDIT-{SHA256(token_id + stablecoin + amount + wallet_from + wallet_to)}
```

This hash is traceable back to the full ATF-AI provenance chain — connecting every on-chain settlement to its compliance attestation.

---

## Tech Stack

- **Rust** + **Axum** — high-performance async HTTP server
- **PostgreSQL** + **SQLx** — double-entry accounting ledger
- **Sled** — embedded key-value store for local state
- **JWT** — stateless authentication
- **ethers-rs** — Ethereum integration
- **TRON** — TRON network integration
- **CCTP** — Circle Cross-Chain Transfer Protocol

---

## Related

- [ATF-AI Protocol](https://github.com/agronetlabs/ATF-AI) — Autonomous Trust Framework
- [ERC-8040 Ecosystem](https://github.com/agronetlabs/erc-8040-ecosystem) — ESG Token Standard
- [CEXS.io](https://cexs.io) — Institutional Exchange (Q2 2026)
- [AgroNet Labs](https://agronet.ai) — Company

---

**AgroNet Labs LLC** | San Francisco | [agronet.ai](https://agronet.ai) | admin@agronet.io
