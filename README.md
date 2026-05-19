[![AI Co-Pilot: OpenClaw](https://img.shields.io/badge/AI%20Co--Pilot-OpenClaw-FF4500?style=for-the-badge&logo=github)](https://openclaw.ai)
# AgroNet Backend â€” Settlement & Liquidity Infrastructure

> Institutional-grade settlement engine built in Rust. Part of the [ATF-AI](https://github.com/agronetlabs/ATF-AI) ecosystem.

[![ATF-AI Verified](https://img.shields.io/badge/ATF--AI-VERIFIED-2ea44f?style=for-the-badge)](https://github.com/agronetlabs/ATF-AI)
[![ERC-8040](https://img.shields.io/badge/ERC--8040-Compliant-0066ff?style=for-the-badge)](https://github.com/agronetlabs/erc-8040-ecosystem)
[![SWIFT ISO 20022](https://img.shields.io/badge/SWIFT-ISO%2020022-orange?style=for-the-badge)]()
[![PWA](https://img.shields.io/badge/PWA-Installable-purple?style=for-the-badge)]()
[![Launch](https://img.shields.io/badge/Launch-Q2%202026-red?style=for-the-badge)]()

[![ISO 20022 Compatible](https://img.shields.io/badge/ISO%2020022-Compatible-00a651?style=for-the-badge&logo=swift&logoColor=white)](https://www.iso20022.org/)
[![SWIFT Ready](https://img.shields.io/badge/SWIFT-Ready-ff6600?style=for-the-badge&logo=swift&logoColor=white)](https://www.swift.com/)
[![ATF-AI Adapter](https://img.shields.io/badge/ATF--AI-ADAPTER-2ea44f?style=for-the-badge&logo=vercel)](https://github.com/agronetlabs/ATF-AI)
[![Provenance Traceable](https://img.shields.io/badge/PROVENANCE-SIGNED-0f9d58?style=for-the-badge&logo=oci)](https://github.com/agronetlabs/ATF-AI)
[![Copilot](https://img.shields.io/badge/GitHub%20Copilot-Active-0066ff?style=for-the-badge&logo=githubcopilot)](https://github.com/features/copilot)
[![OpenAI Codex](https://img.shields.io/badge/OpenAI%20Codex-Active-ff6600?style=for-the-badge&logo=openai&logoColor=white)](https://github.com/features/copilot)

[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/esg-tokenization-protocol)](https://opensource.org/licenses)
![Build](https://img.shields.io/badge/build-passing-brightgreen)
![Status](https://img.shields.io/badge/project-Verified%20Blockchain%20Infra-orange)
![Deployed](https://img.shields.io/badge/deployed-Cloudflare-orange)
![Deployed](https://img.shields.io/badge/deployed-OpenAI-black)

---

## What This Is

The AgroNet Backend is the settlement and liquidity engine powering **CEXS.io** â€” an institutional-grade digital asset exchange built natively on the ATF-AI Autonomous Trust Framework.

Every settlement operation automatically generates an **ATF-AI audit hash** â€” a cryptographic proof of provenance embedded in every transaction.

---

## âœ… Proof of Build

### Settlement Running Live

![Settlement Proof](assets/build-proof-settlement.jpg)

`POST /api/settlement/pull_liquidity` responding with USDT settlement, ATF-AI audit hash generated automatically.

### 10/10 Tests Passing â€” Server Running

![Build Proof](assets/build-proof-tests.jpg)

Clean Rust build, all unit tests passing, server live on port 8080.

---

## Architecture

```
CEXS.io PWA (Frontend)
        â†“
AgroNet Backend (This repo â€” Rust/Axum)
        â†“
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚  Settlement Layer                 â”‚
â”‚  â”œâ”€â”€ TRON (USDT/USDC)            â”‚
â”‚  â”œâ”€â”€ Ethereum (ERC-20)            â”‚
â”‚  â””â”€â”€ CCTP (Cross-Chain)          â”‚
â”œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  ATF-AI Compliance Layer          â”‚
â”‚  â””â”€â”€ Audit hash on every tx       â”‚
â”œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚  Liquidity Layer                  â”‚
â”‚  â””â”€â”€ Pull liquidity from pools    â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
        â†“
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

This hash is traceable back to the full ATF-AI provenance chain â€” connecting every on-chain settlement to its compliance attestation.

---

## Tech Stack

- **Rust** + **Axum** â€” high-performance async HTTP server
- **PostgreSQL** + **SQLx** â€” double-entry accounting ledger
- **Sled** â€” embedded key-value store for local state
- **JWT** â€” stateless authentication
- **ethers-rs** â€” Ethereum integration
- **TRON** â€” TRON network integration
- **CCTP** â€” Circle Cross-Chain Transfer Protocol

---

## Related

- [ATF-AI Protocol](https://github.com/agronetlabs/ATF-AI) â€” Autonomous Trust Framework
- [ERC-8040 Ecosystem](https://github.com/agronetlabs/erc-8040-ecosystem) â€” ESG Token Standard
- [CEXS.io](https://cexs.io) â€” Institutional Exchange (Q2 2026)
- [AgroNet Labs](https://agronet.ai) â€” Company

---

**AgroNet Labs LLC** | San Francisco | [agronet.ai](https://agronet.ai) | admin@agronet.io

