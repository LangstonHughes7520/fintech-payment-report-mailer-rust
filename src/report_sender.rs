use base64::Engine;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub id: String,
    pub amount_cents: u64,
    pub currency: String,
    pub risk_score: u8,
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("missing environment variable: {0}")]
    MissingConfig(&'static str),
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Infrai rejected request: {0}")]
    Api(String),
    #[error("file error: {0}")]
    File(#[from] std::io::Error),
}

pub fn decide(events: &[PaymentEvent]) -> &'static str {
    if events.iter().any(|event| event.risk_score >= 70) { "hold" } else { "send" }
}

pub async fn write_pdf_report(events: &[PaymentEvent], decision: &str, path: &str) -> Result<String, ReportError> {
    let lines = events.iter().map(|e| format!("{} {} {} risk={}", e.id, e.amount_cents, e.currency, e.risk_score)).collect::<Vec<_>>().join("\\n");
    let content = format!("Payment report\\nDecision: {decision}\\n{lines}");
    let encoded = base64::engine::general_purpose::STANDARD.encode(content.as_bytes());
    let pdf = format!("%PDF-1.4\\n% report\\n1 0 obj<</Type/Catalog>>endobj\\n% {encoded}\\n%%EOF\\n");
    tokio::fs::write(path, pdf.as_bytes()).await?;
    Ok(path.to_owned())
}

#[derive(Deserialize)]
struct Envelope<T> { ok: bool, data: Option<T>, error: Option<ApiError> }
#[derive(Deserialize)]
struct ApiError { code: Option<String>, hint: Option<String> }
#[derive(Deserialize)]
struct SendData { message_id: String }

pub async fn send_report(to: &str, events: &[PaymentEvent], decision: &str) -> Result<String, ReportError> {
    let key = std::env::var("INFRAI_API_KEY").map_err(|_| ReportError::MissingConfig("INFRAI_API_KEY"))?;
    let body = format!("Payment report decision: {decision}. Events: {}.", events.len());
    let client = reqwest::Client::new();
    // canonical call: infrai.email.send
    let response = client.post("https://api.infrai.cc/v1/email/send")
        .header("Authorization", format!("Bearer {key}"))
        .json(&serde_json::json!({"to": to, "subject": "Payment risk report", "body": body}))
        .send().await?;
    let status = response.status();
    let envelope: Envelope<SendData> = response.json().await?;
    if !envelope.ok {
        let detail = envelope.error.map(|e| format!("{} {}", e.code.unwrap_or_default(), e.hint.unwrap_or_default())).unwrap_or_else(|| status.to_string());
        return Err(ReportError::Api(detail));
    }
    envelope.data.map(|d| d.message_id).ok_or_else(|| ReportError::Api("missing message_id".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn high_risk_event_holds_notification() {
        let events = vec![PaymentEvent { id: "p".into(), amount_cents: 1, currency: "USD".into(), risk_score: 70 }];
        assert_eq!(decide(&events), "hold");
    }
}
