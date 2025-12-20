# System Design & Philosophy

## Introduction

When I picked up this assignment, I didn't want to just "make it work." I wanted to build a system I'd be comfortable deploying to production on a Friday afternoon. This document explains the reasoning behind the architectural choices in Dodo Payments.

## 1. Architecture: Keeping it Clean

I chose **Hexagonal Architecture** (or Ports & Adapters) for this project.

### Why?
Frameworks like Axum are great, but they change. Databases change. Business logic shouldn't. By isolating the core `domain` logic from the `infrastructure` (Postgres, HTTP clients) and `presentation` (API), I ensured that:

1.  **Testing is Trivia:** I can test the money transfer logic without spinning up a database.
2.  **Clarity:** When you look at `src/domain`, you see *what* the app does (Accounts, Transactions), not *how* it does it (SQL, JSON).

```
[ Presentation Layer (API) ] -> [ Application Layer (Services) ] -> [ Domain Layer (Core) ]
                                          ^
                                          |
                                [ Infrastructure Layer (DB) ]
```

*Note: The Infrastructure layer depends on the Domain layer (by implementing its traits), effectively "inverting" the dependency.*

## 2. Data Integrity: Money is Serious Business

### No Floats Allowed
You'll notice I used `rust_decimal` everywhere. Floating-point math is fine for graphics, but `0.1 + 0.2 != 0.3` is a nightmare for finance. I implemented strict type safety to prevent rounding errors.

### Atomic Transactions
Money cannot be created or destroyed (except by `Deposit` and `Withdraw`). I used **PostgreSQL Transactions** (`START TRANSACTION` ... `COMMIT`) to ensure that a transfer is all-or-nothing. If we debit Alice but fail to credit Bob, the database rolls back. No phantom money.

## 3. Resilience: Distributed Systems are hard

### Idempotency
"Exactly-once" delivery is a myth, but we can fake it.
I added an `idempotency_key` to all transaction requests. If a client retries a request (because your wifi blipped), the server sees the key, realizes it already did that work, and returns the *original* receipt instead of charging you twice.

### Rate Limiting
To keep the service reliable for everyone, I limited requests by IP using the `Governor` algorithm. It’s a "Token Bucket" approach—you get a bucket of tokens that refills over time. Burst usage is allowed, but sustained abuse gets cut off.

## 4. Security

### Webhook Signatures
Sending money is easy; proving you sent it is hard.
When we fire a webhook to tell you "Transfer Complete," we sign the payload with `HMAC-SHA256` using your secret key. This lets you mathematically verify that the message came from us and wasn't tampered with by a man-in-the-middle.

## 5. Why Rust?

Rust's type system forces me to handle edge cases *now*, not at 3 AM when the pager goes off.
- `Result<T, E>` means I can't accidentally ignore an error.
- `Option<T>` means `null` pointer exceptions are impossible.
- `Async/Await` means the server can handle thousands of concurrent requests without eating all the RAM.

---
*Built with ❤️ and `cargo`.*
