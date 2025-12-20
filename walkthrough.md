# Dodo Payments Assignment - Implementation Walkthrough

Hi! Here is a rundown of how I implemented the Payment Processing API. My goal was to build a system that is not just "functional", but robust, testable, and ready for production scenarios.

## 🏗️ Architecture & Design Decisions

I chose a **Hexagonal Architecture** (Ports & Adapters) for this project.
*   **Why?** I wanted to keep the core business logic (`domain`) completely isolated from external concerns like the database or HTTP framework.
*   **The Benefit:** This made testing a breeze. I could unit test complex logic by just mocking the repositories, without needing a running Postgres instance.

The project is structured into four main layers:
1.  **Domain**: The heart of the app. Pure Rust. Contains entities like `Account`, `Transaction`, and the `Repository` traits.
2.  **Application**: The "orchestrator". Services like `TransactionService` live here. They take generic DTOs, validate them, and call the domain.
3.  **Infrastructure**: The "plumbing". This is where I implemented the Postgres repositories using `sqlx` and the Webhook dispatcher using `reqwest`.
4.  **Presentation**: The API layer. I used `Axum` here because it's fast, ergonomic, and integrates beautifully with `tokio`.

## 🚀 Key Features

### 1. Robust Payment Processing
The `TransactionService` handles Deposits, Withdrawals, and Transfers.
*   **Atomicity**: Every operation is wrapped in a database transaction. If any step fails, the whole thing rolls back. No partial state.
*   **Immutable Ledger**: Transactions are never modified after creation. We only append new records.

### 2. Reliability Mechanisms
I didn't want this to be just a "happy path" demo, so I added:
*   **Idempotency**: Clients can safely retry requests (e.g., due to network timeouts) by sending an `idempotency_key`. I check this key before processing to ensure we executed a transaction exactly once.
*   **Concurrency Safety**: I utilized Postgres row-level locking (implicit in updates) and Verified via a stress test that spinning up 10 competing threads transferring money doesn't lose a single cent.

### 3. Asynchronous Webhooks with Security
When a payment processing finishes, we can't block the API response while notifying user webhooks.
*   **Implementation**: I used `tokio::spawn` to fire-and-forget the webhook dispatching.
*   **Security**: To prove the webhook came from us, I sign every payload using **HMAC-SHA256**. The signature is sent in the `X-Dodo-Signature` header.
*   **Retries**: If the user's server flakiness, I implemented a jittered exponential backoff retry mechanism.

### 4. Production-Ready Controls
*   **Rate Limiting**: Used `governor` to implement an IP-based token bucket. No single IP can flood our service.
*   **Authentication**: Custom middleware validates API keys (hashed in DB) before allowing access to sensitive routes.
*   **Observability**: Hooked up `tracing` for structured logs. You can see exactly what's happening in every request.

## 🛠️ Tech Stack Validation

I verified the entire system with a comprehensive test suite (`cargo test`).

*   **Unit Tests**: Checked domain rules (e.g., Money arithmetic).
*   **Integration Tests**: Spun up the full app and ran end-to-end flows (Create Account -> Deposit -> Transfer).
*   **Result**: **37/37 tests passed**, with 0 compiler warnings.

## 🏁 Ready to Run

The app is containerized. Just run:
```bash
docker-compose up --build
```
And check the health:
```bash
curl http://localhost:8080/health
```

I'm pretty happy with how clean the final codebase turned out. Let me know if you have any questions!
