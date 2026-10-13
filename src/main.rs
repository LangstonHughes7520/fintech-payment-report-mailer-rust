mod report_sender;

use report_sender::{decide, PaymentEvent, ReportError};

#[tokio::main]
async fn main() -> Result<(), ReportError> {
    let recipient = std::env::var("REPORT_TO").map_err(|_| ReportError::MissingConfig("REPORT_TO"))?;
    let events = vec![
        PaymentEvent { id: "pay_1001".into(), amount_cents: 12500, currency: "USD".into(), risk_score: 18 },
        PaymentEvent { id: "pay_1002".into(), amount_cents: 4900, currency: "USD".into(), risk_score: 22 },
    ];
    let decision = decide(&events);
    let path = report_sender::write_pdf_report(&events, &decision, "payment-report.pdf").await?;
    println!("report={path} decision={decision}");
    if decision == "hold" {
        println!("notification skipped: review required");
        return Ok(());
    }
    let message_id = report_sender::send_report(&recipient, &events, &decision).await?;
    println!("message_id={message_id}");
    Ok(())
}
