use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox};
use std::{env, time::Duration};

#[derive(Clone)]
pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl EmailService {
    pub fn from_env() -> Result<Self, String> {
        let host = env::var("MAIL_HOST").map_err(|_| "MAIL_HOST must be set".to_string())?;
        let port = env::var("MAIL_PORT")
            .map_err(|_| "MAIL_PORT must be set".to_string())?
            .parse::<u16>()
            .map_err(|_| "MAIL_PORT must be a valid port".to_string())?;
        let from = env::var("MAIL_FROM")
            .map_err(|_| "MAIL_FROM must be set".to_string())?
            .parse::<Mailbox>()
            .map_err(|e| format!("MAIL_FROM is invalid: {e}"))?;

        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
            .port(port)
            .timeout(Some(Duration::from_secs(3)))
            .build();

        Ok(Self { transport, from })
    }

    pub async fn send_status_change(
        &self,
        recipient: &str,
        status: &str,
        review_note: Option<&str>,
    ) -> Result<(), String> {
        let recipient = recipient
            .parse::<Mailbox>()
            .map_err(|e| format!("invalid recipient address: {e}"))?;

        let mut body =
            format!("The status of your military service application has changed to: {status}.");
        if let Some(note) = review_note.filter(|note| !note.trim().is_empty()) {
            body.push_str("\n\nReview note:\n");
            body.push_str(note);
        }

        let message = Message::builder()
            .from(self.from.clone())
            .to(recipient)
            .subject(format!("Military Portal application status: {status}"))
            .body(body)
            .map_err(|e| format!("could not build email: {e}"))?;

        self.transport
            .send(message)
            .await
            .map_err(|e| format!("MAIL delivery failed: {e}"))?;

        Ok(())
    }
}
