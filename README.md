# Mutao (木桃)

[![CI](https://github.com/qiaopengjun5162/mutao/actions/workflows/ci.yml/badge.svg)](https://github.com/qiaopengjun5162/mutao/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.87+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-62-green.svg)](#testing)

> "Throw me a peach, I'll repay you with a jade." — *The Book of Songs*

A cash-free barter platform with AI matching and Web3 provenance, built for digital nomads and youth communities.

[中文文档](README_zh.md)

## Tech Stack

| Layer | Technology | Responsibility |
|---|---|---|
| Core Engine | Rust 1.87+ (axum 0.7 + tokio) | API routing, graph matching, state machine |
| Persistence | PostgreSQL 16 (sqlx 0.7) | Items, demands, swap cycles |
| AI Scalpel | Python 3.12+ | Image/text → tags + value tier |
| Web3 Attestation | Solidity | On-chain provenance (multi-chain interface) |
| Frontend | Next.js 16 + shadcn/ui | User interface (Turbopack) |

## Quick Start

```bash
# Option 1: Docker (recommended)
cp .env.example .env   # Edit JWT_SECRET
just docker-up         # Start all services

# Option 2: Local development
# Prerequisites: PostgreSQL running, Rust 1.87+
cp .env.example .env   # Edit DATABASE_URL
just db-init           # Initialize database
cargo run              # Start backend at http://localhost:3000

# Frontend
cd frontend && pnpm install && pnpm run dev   # http://localhost:3001
```

## API Endpoints

| Method | Path | Description |
|---|---|---|
| GET | /api/health | Health check |
| POST | /api/auth/register | User registration |
| POST | /api/auth/login | User login |
| POST | /api/items | Create item |
| GET | /api/items | List items |
| GET | /api/items/:id | Item detail |
| POST | /api/items/:id/match | Trigger matching |
| PATCH | /api/items/:id/status | Update item status |
| POST | /api/items/analyze | AI tag extraction |
| POST | /api/items/:id/attest | Web3 attestation |
| GET | /api/items/:id/history | On-chain history |
| POST | /api/demands | Create demand |
| GET | /api/demands | List demands |
| GET | /api/cycles | List swap cycles |
| POST | /api/cycles/:id/confirm | Confirm swap |
| GET | /api/ws | WebSocket notifications |

## Project Structure

```
mutao/
├── src/
│   ├── main.rs          # axum routes + handlers
│   ├── lib.rs           # AppState + library entry
│   ├── models.rs        # Domain models
│   ├── matcher.rs       # DFS multi-node cycle discovery
│   ├── store.rs         # Data access layer
│   ├── auth.rs          # JWT authentication
│   ├── error.rs         # Unified error handling
│   ├── ws.rs            # WebSocket broadcast hub
│   └── blockchain/      # Multi-chain adapters
├── frontend/src/app/    # Next.js pages (9 routes)
├── scalpel/             # Python AI scalpel
├── contracts/           # Solidity attestation contract
├── migrations/          # Database migrations
├── tests/               # Rust tests (59)
├── Dockerfile           # Rust backend image (multi-stage)
├── Dockerfile.scalpel   # Python scalpel image
└── docker-compose.yml   # Backend + DB + Scalpel orchestration
```

## Testing

```bash
cargo nextest run             # Rust tests (59)
cd scalpel && pytest -v       # Python tests (10)
just test-all                 # All tests
cargo llvm-cov nextest --html # Coverage report
```

## Core Algorithm

`matcher::Matcher::find_cycles()` uses DFS to discover swap cycles of length 2-4 in a directed graph:

1. **Build edges**: `i → j` when `Demand[i].offer_tags` intersects `Demand[j].target_tags`
2. **DFS**: From each node, search for a directed cycle back to the start
3. **Filter**: Deduplicate, remove same-user cycles, validate cycle closure

## License

MIT
