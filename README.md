# Dodo Payments Assignment

Hey there! 👋 This is my implementation of the **Transaction Service** for the Dodo Payments assignment. I've built this in Rust using `Axum` and `SQLx`, focusing heavily on correctness, type safety, and clean architecture.

## What's Inside?

This isn't just a CRUD app; I wanted to build something that feels production-ready.
- **Hexagonal Architecture:** The core logic (`domain`) is completely isolated from the database and API. This makes unit testing the business rules super easy (and fast!).
- **Acid Transactions:** Money transfers between accounts happen atomically. If any part of a transfer fails, the whole thing rolls back. No partial states here.
- **Type-Safe Money:** I'm using `rust_decimal` for all currency calculations. Floating point math is great for games, but terrible for money, so strictly no `f64` here.
- **Secure Auth:** API keys are hashed (SHA-256) before hitting the DB. Even if the database leaks, the keys are safe.
- **Dockerized Setup:** PostgreSQL and Liquibase migrations run in containers, ensuring the environment is exactly the same on your machine as it is on mine.

## Tech Stack
- **Language:** Rust (2024 edition)
- **Web Framework:** Axum 0.8
- **Database:** PostgreSQL 16 (accessed via SQLx)
- **Migrations:** Liquibase
- **Runtime:** Tokio

## Getting Started

### Prerequisites
You'll need `docker`, `docker-compose`, and `cargo` installed.

### Running it
I've wrapped the common commands in a `Makefile` to save some typing:

1. **Spin up the infrastructure:**
   ```bash
   make up
   # This starts Postgres, runs migrations, and starts the app container.
   ```

2. **Run locally (for development):**
   If you prefer running the Rust binary on your host machine while keeping the DB in Docker:
   ```bash
   make dev
   # Waits for DB to be ready, then you can run:
   export DATABASE_URL=postgres://postgres:postgres@localhost:5432/dodo-payments
   cargo run
   ```

3. **Run Tests:**
   ```bash
   make test
   ```

## Using the API

The server listens on `http://localhost:8080`. Here are a few endpoints to verify things work.

### 1. Create an Account (Public)
```bash
curl -X POST http://localhost:8080/accounts \
  -H "Content-Type: application/json" \
  -d '{"business_name": "Dodo Coffee Shop"}'
```

### 2. Verify Auth (Protection Check)
Try listing accounts without a key – you should get a 401.
```bash
curl -v http://localhost:8080/accounts
```

### 3. Setup Authenticated Access (Manual Step)
Since the "Generate Key" endpoint is hypothetical for this assignment, we insert a key manually. The system expects a **SHA-256 hash** of the key.

**Step A: Generate the Hash**
Run this python one-liner to hash the key `my_secret_key`:
```bash
python3 -c "import hashlib; print(hashlib.sha256(b'my_secret_key').hexdigest())"
# Output: 31f7a65e315586ac198bd798b6629ce4903d0899476d5741a69a8715403079aa
```

**Step B: Insert into Database**
Execute this SQL command (replace `<YOUR_ACCOUNT_ID>` with the UUID from Step 1):
```bash
# Using docker-compose to run psql
docker-compose exec -T -e PGPASSWORD=postgres postgres psql -U postgres -d dodo-payments -c "
INSERT INTO api_keys (id, key_hash, account_id, rate_limit_per_hour, created_at)
VALUES (
    gen_random_uuid(),
    '31f7a65e315586ac198bd798b6629ce4903d0899476d5741a69a8715403079aa',
    '<YOUR_ACCOUNT_ID>',
    1000,
    now()
);"
```

### 4. Deposit Money (Protected)
```bash
curl -X POST http://localhost:8080/transactions/deposit \
  -H "x-api-key: my_secret_key" \
  -H "Content-Type: application/json" \
  -d '{
    "account_id": "<YOUR_ACCOUNT_UUID>",
    "amount": 100.00
  }'
```

## Design Decisions

- **Why UUIDs?** They are safer for distributed systems and prevent ID enumeration attacks compared to sequential integers.
- **Why no ORM?** I prefer `SQLx` because it gives me compile-time verification of my SQL queries. If I typo a column name, the code won't compile. It's magical. 
- **Soft Deletes:** Deleting financial data is scary. I used a `deleted_at` column so we can essentially "archive" accounts without losing history.

## What's Next?
Day 3 is all about **Webhooks**. I plan to implement a dispatcher that notifies external URLs when transactions complete, complete with HMAC signatures so clients can trust the events came from us.

Enjoy reviewing! Let me know if you have any questions. 🚀
