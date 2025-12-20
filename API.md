# API Reference

Welcome to the Dodo Payments API.

Base URL: `http://localhost:8080`

## Authentication

We use **API Keys** to keep things secure.
Pass your key in the `x-api-key` header.

```bash
x-api-key: your_secret_key
```

> **Note:** Failed auth returns `401 Unauthorized`.

---

## Accounts

### Create Account
Public endpoint. Open your doors for business.

- **POST** `/accounts`

```bash
curl -X POST http://localhost:8080/accounts \
  -H "Content-Type: application/json" \
  -d '{"business_name": "Alice Corp"}'
```

**Response (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "business_name": "Alice Corp",
  "balance": "0",
  "webhook_secret": "generated_secret_...",
  "created_at": "2024-12-20T10:00:00Z"
}
```

### Get Account
Check your balance.

- **GET** `/accounts/:id`
- **Auth:** Required

```bash
curl -H "x-api-key: secret" http://localhost:8080/accounts/550e8400...
```

---

## Transactions

### Deposit
Add funds to an account.

- **POST** `/transactions/deposit`
- **Auth:** Required

```bash
curl -X POST http://localhost:8080/transactions/deposit \
  -H "x-api-key: secret" \
  -H "Content-Type: application/json" \
  -d '{
    "account_id": "550e8400...",
    "amount": 1000.50,
    "idempotency_key": "unique_req_123"
  }'
```

| Field | Type | Description |
|-------|------|-------------|
| `amount` | Decimal | The amount to deposit. Must be positive. |
| `idempotency_key` | String | (Optional) Unique ID to prevent double-charging on retry. |

### Transfer
Move money between accounts safely.

- **POST** `/transactions/transfer`
- **Auth:** Required

```bash
curl -X POST http://localhost:8080/transactions/transfer \
  -H "x-api-key: secret" \
  -H "Content-Type: application/json" \
  -d '{
    "from_account_id": "550e8400...",
    "to_account_id": "661f9500...",
    "amount": 50.00
  }'
```

---

## Webhooks

We'll hit your URL when cool stuff happens.

### Register a Webhook
Tell us where to send the events.

- **POST** `/webhooks/register`
- **Auth:** Required

```json
{
  "url": "https://api.alice-corp.com/hooks",
  "event": "transaction.completed"
}
```

### Verifying Signatures
Don't trust blindly. Verify the `x-webhook-signature` header.

1. Take the raw request body.
2. Calculate `HMAC-SHA256(body, your_webhook_secret)`.
3. Compare it to the header value (format: `sha256=<hex_hash>`).
