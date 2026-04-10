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

impl BlameEmail {
    /// If `demo_addr` is `Some`, redirect the email: send only to that
    /// address with no CC recipients.  The original `to` is preserved in
    /// the subject for context.
    pub fn apply_demo_override(mut self, demo_addr: Option<&str>) -> Self {
        if let Some(addr) = demo_addr {
            self.to = addr.to_string();
            self.cc.clear();
        }
        self
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_email() -> BlameEmail {
        BlameEmail {
            to: "blamee@example.com".into(),
            cc: vec!["manager@example.com".into(), "team@example.com".into()],
            subject: "Code Quality Concern".into(),
            body: "Dear Alice, please explain.".into(),
        }
    }

    #[test]
    fn demo_override_redirects_to_demo_address() {
        let email = sample_email().apply_demo_override(Some("demo@example.com"));
        assert_eq!(email.to, "demo@example.com");
        assert!(email.cc.is_empty());
        assert!(email.subject.contains("[demo — would go to blamee@example.com]"));
    }

    #[test]
    fn demo_override_preserves_original_subject() {
        let email = sample_email().apply_demo_override(Some("demo@example.com"));
        assert!(email.subject.starts_with("Code Quality Concern"));
    }

    #[test]
    fn demo_override_does_not_alter_body() {
        let email = sample_email().apply_demo_override(Some("demo@example.com"));
        assert_eq!(email.body, "Dear Alice, please explain.");
    }

    #[test]
    fn no_demo_override_leaves_email_unchanged() {
        let email = sample_email().apply_demo_override(None);
        assert_eq!(email.to, "blamee@example.com");
        assert_eq!(email.cc.len(), 2);
        assert_eq!(email.subject, "Code Quality Concern");
    }
}
