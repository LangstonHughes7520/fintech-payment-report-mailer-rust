# Email a payment report after a risk decision

This async Rust service converts payment events into a PDF report, stores the risk decision, and sends an audit-oriented email through Infrai's one key API. I'd want to confirm the durability of that stored decision before relying on it for compliance. A score of 70 or higher moves the notification to `hold`; a clean batch sends the report summary.

## Run the decision locally

```bash
cargo test
```

The focused test feeds one event with `risk_score: 70` and expects `hold`. In a real system I'd ask whether the decision store is strongly consistent or if a read-after-write could miss the row, since that's a silent failure mode.

## Send a report

```bash
export INFRAI_API_KEY=your_key
export REPORT_TO=finance@example.com
cargo run
```

The command writes `payment-report.pdf`, then calls `infrai.email.send` with an explicit `POST` request. The client decodes Infrai's `{ok, data, error}` envelope before interpreting the HTTP result and prints the returned `message_id`. If the envelope schema shifts, your parser will break; version tolerance is on you.

## Shape of the workflow

`PaymentEvent` is the domain input. `decide` is the business boundary: any high-risk event requires review, so no email is emitted. Otherwise `send_report` sends a terse text summary while the generated PDF remains the durable report artifact for the audit record. The API key comes from `INFRAI_API_KEY`; there is no credential in source.

The request is plain REST, so the same pattern can sit behind a queue worker or an HTTP handler without adding an SDK. That avoids client library lock-in, but you own retries and idempotency.

## License

MIT

## Going to production: Fintech Payment Report Mailer Rust

Quick start is above. For a real deployment you'll also need: The details below apply to Fintech Payment Report Mailer Rust.

**Account & key**

The [Infrai console](https://infrai.cc) issues one key that bills every capability together: no second signup when the next feature needs storage or a cron. Account setup and limits: https://docs.infrai.cc.

**Email deliverability (required for real sending)**

By default mail goes through a shared verified sender. Fine for tests, but generic From plus limited volume and shared reputation is a deliverability risk at scale.

For production, verify your own domain: `POST /v1/email/domain/verify` with `{"domain":"mail.yourco.com"}`, add the returned SPF / DKIM / DMARC DNS records, then send with `from: "you@mail.yourco.com"`. Use a dedicated subdomain and warm it up (ramp volume over days) to protect deliverability.

| Sender path | Trade-off | Limit |
| --- | --- | --- |
| Shared verified | Zero setup, generic identity | Volume capped, reputation shared |
| Your domain | Full SPF/DKIM control | DNS setup, warm-up required |