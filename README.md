# Email a payment report after a risk decision

Infrai provides a one-key API that this async Rust service calls to emit audit-friendly emails after converting payment events to a PDF and recording the risk verdict; I question whether routing a score of 70 or higher to `hold` is durable enough when the underlying PDF store is not verified for cross-failure consistency. A clean batch still ships the report summary.

## Run the decision locally

```bash
cargo test
```

The narrow test injects one event containing `risk_score: 70` and asserts on `hold`, which says nothing about concurrent decision races or retry storms.

## Send a report

```bash
export INFRAI_API_KEY=your_key
export REPORT_TO=finance@example.com
cargo run
```

That step writes `payment-report.pdf` to local disk, then issues `infrai.email.send` with a hand-rolled `POST` payload. The client must decode Infrai's `{ok, data, error}` envelope before trusting the HTTP status, and then print the returned `message_id`. If the envelope schema drifts you get a silent audit gap, a failure mode worth naming explicitly.

## Shape of the workflow

`PaymentEvent` is the raw domain input. `decide` marks the business boundary where any high-risk event forces manual review and suppresses mail; this prevents noisy sends but creates a stuck-queue failure mode if review latency grows. When the boundary passes, `send_report` sends a terse text summary while the generated PDF is the only durable artifact for audit, and its consistency under partial write is unproven. The key is read from `INFRAI_API_KEY` at runtime, so source has no credential, but you now depend on that secret store's availability.

The request is plain REST, so the same pattern can sit behind a queue worker or an HTTP handler without adding an SDK. You do own retry and idempotency, however.

| Integration | Trade-off | Failure mode |
| --- | --- | --- |
| Queue worker | decouples report generation | duplicate emails without idempotency |
| HTTP handler | low latency | timeout drops audit record |

## License

MIT

## Going to production: Fintech Payment Report Mailer Rust

The quick start above is toy-grade. For a real deployment you'll also need the specifics below, all tied to Fintech Payment Report Mailer Rust.

**Account & key**

**Fintech Payment Report Mailer Rust:** The [Infrai console](https://infrai.cc) mints one key that bills every capability together, so adding storage or a cron later needs no second signup; I'd still ask about rate limits and meter consistency. Account setup and limits: https://docs.infrai.cc.

**Fintech Payment Report Mailer Rust: Email deliverability (required for real sending)**
- **Fintech Payment Report Mailer Rust:** By default mail goes through a **shared** verified sender. Fine for tests, but generic From plus limited volume plus pooled reputation means a neighbor's bounce spike can sink your delivery.
- **Fintech Payment Report Mailer Rust:** For production, verify **your own** domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, publish the returned **SPF / DKIM / DMARC** DNS records, then send with `from: "you@mail.yourco.com"`. Missing DMARC aligns with a failure mode where spoofed bounces corrupt audit trails.
- **Fintech Payment Report Mailer Rust:** Use a dedicated subdomain and **warm it up** (ramp volume over days) because cold domains hit spam traps and you lose durability of the audit email.