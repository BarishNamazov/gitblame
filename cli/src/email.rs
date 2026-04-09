use crate::env_config::EnvConfig;
use crate::error::Result;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

/// All the pieces of a blame email, ready to send.
#[derive(Debug, Clone)]
pub struct BlameEmail {
    /// Recipient address.
    pub to: String,
    /// CC list.
    pub cc: Vec<String>,
    /// Email subject line.
    pub subject: String,
    /// Plain-text body.
    pub body: String,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Synchronous SMTP email client backed by lettre.
pub struct EmailClient {
    transport: SmtpTransport,
    from: Mailbox,
}

impl EmailClient {
    /// Build a new [`EmailClient`] from environment configuration.
    pub fn new(config: &EnvConfig) -> Result<Self> {
        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );

        let transport = match config.smtp_encryption.as_str() {
            "tls" => SmtpTransport::relay(&config.smtp_host)
                .map_err(|e| format!("SMTP relay error: {e}"))?
                .port(config.smtp_port)
                .credentials(creds)
                .build(),
            // default to STARTTLS
            _ => SmtpTransport::starttls_relay(&config.smtp_host)
                .map_err(|e| format!("SMTP STARTTLS relay error: {e}"))?
                .port(config.smtp_port)
                .credentials(creds)
                .build(),
        };

        let from: Mailbox = config
            .smtp_from
            .parse()
            .map_err(|e| format!("Invalid SMTP_FROM address \"{}\": {e}", config.smtp_from))?;

        Ok(Self { transport, from })
    }

    /// Send a plain-text email.
    pub fn send_email(
        &self,
        to: &str,
        cc_list: &[String],
        subject: &str,
        body: &str,
    ) -> Result<()> {
        let to_mbox: Mailbox = to
            .parse()
            .map_err(|e| format!("Invalid recipient \"{to}\": {e}"))?;

        let mut builder = Message::builder()
            .from(self.from.clone())
            .to(to_mbox)
            .subject(subject);

        for cc in cc_list {
            let cc_mbox: Mailbox = cc
                .parse()
                .map_err(|e| format!("Invalid CC address \"{cc}\": {e}"))?;
            builder = builder.cc(cc_mbox);
        }

        let message = builder
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {e}"))?;

        self.transport
            .send(&message)
            .map_err(|e| format!("Failed to send email: {e}"))?;

        Ok(())
    }

    /// Convenience wrapper for sending a [`BlameEmail`].
    pub fn send_blame_email(&self, email: &BlameEmail) -> Result<()> {
        self.send_email(&email.to, &email.cc, &email.subject, &email.body)
    }
}
