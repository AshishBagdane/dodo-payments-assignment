# Transaction Service - Problem Statement & Development Plan

## Original Assignment Requirements

### Core Requirements (MUST HAVE)
1. ✅ **API Authentication**: Secure API key-based authentication
2. ✅ **Accounts**: Create accounts + check balances
3. ✅ **Transactions**: Credit, Debit, Transfer with atomic balance updates
4. ✅ **Webhooks**: Reliable and secure transaction notifications
5. ✅ **Database**: PostgreSQL for persistence (accounts, transactions, api_keys, webhooks)
6. ✅ **API Documentation**: Clear request/response/error documentation
7. ✅ **Docker Compose**: One-command setup (`docker compose up`)

### Bonus Features (INCLUDE 2-3)
1. ✅ **Idempotency Keys**: Critical for payment reliability - INCLUDE
2. ✅ **Rate Limiting**: Per API key - INCLUDE
3. ✅ **Basic Logging**: Using tracing crate - INCLUDE
4. ❌ **OpenTelemetry**: Skip (time-intensive, basic logging sufficient)

### Deliverables
1. ✅ GitHub repository with source code
2. ✅ DESIGN.md (architecture, schemas, trade-offs)
3. ✅ API.md (endpoints, request/response formats)
4. ✅ README.md (setup + example requests)
5. ✅ Docker Compose configuration

### Deadline
**December 21, 2025 (Sunday) - 23:59 IST** (4 days from today)

---

## 4-Day Development Plan

### **DAY 1 (Dec 17)**: Foundation + Domain
**Goal: Database working, domain models complete**

| Unit | Component | Time | Deliverable |
|------|-----------|------|-------------|
| 1 | Project Init | 30min | Cargo.toml workspace |
| 2 | DB Schema | 45min | Schema design (SQL) |
| 3 | Docker Compose | 30min | PostgreSQL + app container |
| 4 | Migrations | 45min | SQLx migration files |
| 5 | Config | 30min | Environment-based config |
| 6 | Error Types | 45min | Domain error hierarchy |
| 7 | Account Entity | 45min | Account struct + validation |
| 8 | Transaction Entity | 45min | Transaction types (Credit/Debit/Transfer) |
| 9 | Money ValueObject | 30min | Decimal-based Money type |
| 10 | Repository Traits | 45min | Interface definitions |

**End-of-Day Checkpoint**: Can create accounts and transactions in database

---

### **DAY 2 (Dec 18)**: Infrastructure + API
**Goal: All REST endpoints working with authentication**

| Unit | Component | Time | Deliverable |
|------|-----------|------|-------------|
| 11 | DB Pool | 30min | SQLx connection pooling |
| 12 | Account Repo | 1hr | PostgreSQL account repository |
| 13 | Transaction Repo | 1.5hr | Atomic transaction operations |
| 14 | ApiKey Repo | 45min | Key storage + hashing |
| 15 | Account Service | 1hr | Business logic layer |
| 16 | Transaction Service | 1.5hr | Credit/Debit/Transfer use cases |
| 17 | Axum Server | 45min | HTTP server setup |
| 18 | Account Endpoints | 1hr | POST /accounts, GET /accounts/:id |
| 19 | Transaction Endpoints | 1hr | POST /transactions/* |
| 20 | Auth Middleware | 1hr | API key extraction + validation |

**End-of-Day Checkpoint**: Full CRUD API working with authentication

---

### **DAY 3 (Dec 19)**: Webhooks + Bonus Features
**Goal: Complete feature set operational**

| Unit | Component | Time | Deliverable |
|------|-----------|------|-------------|
| 21 | Webhook Entity | 45min | Webhook registration model |
| 22 | Webhook Repo | 45min | Storage + retrieval |
| 23 | Webhook Endpoints | 1hr | POST/GET/DELETE /webhooks |
| 24 | Webhook Dispatcher | 2hr | Async HTTP delivery |
| 25 | HMAC Signatures | 1hr | Secure webhook signing |
| 26 | Retry Logic | 1hr | Exponential backoff |
| 27 | Idempotency | 1.5hr | Idempotency key middleware |
| 28 | Rate Limiter | 1.5hr | Token bucket per API key |
| 29 | Logging Setup | 1hr | Tracing + structured logs |

**End-of-Day Checkpoint**: All features tested end-to-end

---

### **DAY 4 (Dec 20)**: Documentation + Testing + Submission
**Goal: Production-ready submission**

| Unit | Component | Time | Deliverable |
|------|-----------|------|-------------|
| 30 | Integration Tests | 2hr | Critical path tests |
| 31 | DESIGN.md | 2hr | Architecture documentation |
| 32 | API.md | 1.5hr | API reference with examples |
| 33 | README.md | 1hr | Setup guide + curl examples |
| 34 | Error Handling | 1hr | Consistent error responses |
| 35 | Health Endpoints | 30min | /health, /ready |
| 36 | Docker Polish | 1hr | Final compose validation |
| 37 | Code Cleanup | 1hr | Remove TODOs, format |
| 38 | Final Testing | 1hr | End-to-end verification |
| 39 | Submission | 30min | GitHub push + email |

**Submission Buffer**: Dec 21 morning for final review

---

## Technical Architecture

### Technology Stack
```
Runtime:     Rust 2021 + Tokio async
Framework:   Axum 0.7
Database:    PostgreSQL 16 + SQLx
Security:    SHA256 hashing, HMAC-SHA256
Logging:     tracing + tracing-subscriber
Containers:  Docker + Docker Compose
```

### System Architecture (Hexagonal/Clean)
```
┌─────────────────────────────────────────┐
│         Presentation Layer              │
│  (Axum Handlers + Middleware)           │
│  - REST API Endpoints                   │
│  - Auth Middleware                      │
│  - Rate Limiting                        │
│  - Idempotency Check                    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│        Application Layer                │
│  (Services - Business Logic)            │
│  - AccountService                       │
│  - TransactionService                   │
│  - WebhookService                       │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│          Domain Layer                   │
│  (Entities + Value Objects + Traits)    │
│  - Account, Transaction, ApiKey         │
│  - Money, TransactionType               │
│  - Repository Traits (interfaces)       │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       Infrastructure Layer              │
│  (External Systems)                     │
│  - PostgreSQL Repositories              │
│  - HTTP Webhook Client                  │
│  - Config Loader                        │
└─────────────────────────────────────────┘
```

### Database Schema (Simplified)
```sql
accounts (id, business_name, balance, created_at, updated_at)
transactions (id, type, from_account, to_account, amount, idempotency_key, created_at)
api_keys (id, key_hash, account_id, rate_limit, created_at)
webhooks (id, account_id, url, secret, events[], active, created_at)
webhook_deliveries (id, webhook_id, transaction_id, status, attempts, next_retry)
```

---

## Quality Assurance Strategy

### Code Quality Standards
- ✅ No `unwrap()` or `panic!()` in production code
- ✅ All functions return `Result<T, E>`
- ✅ Compiler warnings = 0
- ✅ Self-documenting code (minimal comments)
- ✅ Functions < 30 lines
- ✅ Modules < 200 lines

### Security Checklist
- ✅ API keys hashed (SHA256) before storage
- ✅ Webhook signatures (HMAC-SHA256)
- ✅ Parameterized SQL queries (no injection)
- ✅ Input validation on all endpoints
- ✅ Rate limiting enforced
- ✅ No secrets in code (env vars only)

### Testing Strategy
- ✅ Integration tests for critical paths
- ✅ Database transaction tests
- ✅ API endpoint tests (happy + error cases)
- ✅ Authentication tests
- ✅ Docker Compose smoke tests

---

## Development Workflow

### Per-Unit Process
1. **Build**: Create minimal working unit
2. **Present**: Show files + verification
3. **Wait**: Get explicit approval
4. **Iterate**: Fix if needed
5. **Proceed**: Move to next unit ONLY after approval

### Quality Gates (Every Unit)
- [ ] Compiles without warnings
- [ ] Follows SOLID principles
- [ ] Error handling complete
- [ ] Can be tested independently
- [ ] Ashish approves

---

## Evaluation Alignment

### How This Meets Criteria

| Criterion | Our Approach |
|-----------|--------------|
| Clarity of design | Hexagonal architecture with clear layer separation |
| Correctness | Atomic transactions, idempotency, retry logic |
| Security | Hashed keys, HMAC signatures, rate limiting |
| Data model | Normalized schema, UUIDs, proper constraints |
| Operational | Structured logging, health checks, Docker |
| Communication | Code-first, documentation on request |

---

## Risk Mitigation

### Time Risks
- **Mitigation**: Focus on core requirements first
- **Buffer**: 1 day before deadline for polish
- **Scope control**: Skip OpenTelemetry (complex)

### Technical Risks
- **Mitigation**: Use proven libraries (Axum, SQLx)
- **Testing**: Verify each unit before proceeding
- **Fallback**: Simplify bonus features if needed

---

## SUCCESS CRITERIA

### Minimum Viable Submission
- ✅ `docker compose up` works first try
- ✅ Can create account via API
- ✅ Can execute all transaction types
- ✅ Balances update atomically
- ✅ Webhooks deliver successfully
- ✅ Authentication blocks unauthorized access
- ✅ Documentation is clear and complete

### Ideal Submission (Target)
- ✅ All above + idempotency working
- ✅ Rate limiting enforced
- ✅ Structured logging throughout
- ✅ Clean, well-organized code
- ✅ Comprehensive test coverage