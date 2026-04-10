use crate::error::Result;

/// Configuration loaded from a `.env` file or environment variables.
///
/// Required keys: `OPENROUTER_API_KEY`, `SMTP_HOST`, `SMTP_USERNAME`,
/// `SMTP_PASSWORD`, `SMTP_FROM`.  Optional: `SMTP_PORT` (default 587),
/// `SMTP_ENCRYPTION` (default "starttls").
#[derive(Debug, Clone)]
pub struct EnvConfig {
    /// API key for OpenRouter.
    pub openrouter_api_key: String,
    /// SMTP server hostname.
    pub smtp_host: String,
    /// SMTP server port (default: 587).
    pub smtp_port: u16,
    /// SMTP authentication username.
    pub smtp_username: String,
    /// SMTP authentication password.
    pub smtp_password: String,
    /// Sender ("From") email address.
    pub smtp_from: String,
    /// SMTP encryption mode: `"starttls"` or `"tls"` (default: `"starttls"`).
    pub smtp_encryption: String,
    /// When set, all emails are sent only to this address instead of the
    /// actual recipients.  Useful for demos.
    pub demo_email_address: Option<String>,
}

impl EnvConfig {
    /// Load configuration from a `.env` file (via dotenvy) with fallback to
    /// the process environment.  Returns an error if any required field is
    /// missing.
    pub fn load() -> Result<Self> {
        // Best-effort load of .env — ignore errors (file may not exist).
        // Honor GITBLAME_DOTENV as an explicit override path.
        match std::env::var("GITBLAME_DOTENV") {
            Ok(path) if !path.is_empty() => {
                let _ = dotenvy::from_path(std::path::Path::new(&path));
            }
            _ => {
                let _ = dotenvy::dotenv();
            }
        }

        let openrouter_api_key = require_var("OPENROUTER_API_KEY")?;
        let smtp_host = require_var("SMTP_HOST")?;
        let smtp_username = require_var("SMTP_USERNAME")?;
        let smtp_password = require_var("SMTP_PASSWORD")?;
        let smtp_from = require_var("SMTP_FROM")?;

        let smtp_port: u16 = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .map_err(|e| format!("SMTP_PORT is not a valid port number: {e}"))?;

        let smtp_encryption = std::env::var("SMTP_ENCRYPTION")
            .unwrap_or_else(|_| "starttls".to_string());

        if smtp_encryption != "starttls" && smtp_encryption != "tls" {
            return Err(format!(
                "SMTP_ENCRYPTION must be \"starttls\" or \"tls\", got \"{smtp_encryption}\""
            )
            .into());
        }

        let demo_email_address = std::env::var("GITBLAME_DEMO_EMAIL_ADDRESS").ok()
            .filter(|v| !v.is_empty());

        Ok(Self {
            openrouter_api_key,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from,
            smtp_encryption,
            demo_email_address,
        })
    }

    /// Returns `true` if all required environment variables are set and valid.
    ///
    /// This is a non-failing convenience check — it will never panic or print
    /// errors.
    pub fn is_available() -> bool {
        Self::load().is_ok()
    }
}

/// Helper: read a required environment variable or return a descriptive error.
fn require_var(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| {
        format!(
            "Required environment variable {name} is not set. \
             Set it in your .env file or export it in your shell."
        )
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_var_gives_helpful_error() {
        // Ensure the var is absent for this test.
        // SAFETY: test is single-threaded; no other thread reads this var.
        unsafe { std::env::remove_var("OPENROUTER_API_KEY") };
        let err = require_var("OPENROUTER_API_KEY").unwrap_err();
        assert!(
            err.to_string().contains("OPENROUTER_API_KEY"),
            "Error should mention the variable name"
        );
    }

    #[test]
    fn is_available_returns_false_when_vars_missing() {
        // Clear required vars to ensure load() fails.
        // SAFETY: test is single-threaded; no other thread reads these vars.
        unsafe {
            std::env::remove_var("OPENROUTER_API_KEY");
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_USERNAME");
            std::env::remove_var("SMTP_PASSWORD");
            std::env::remove_var("SMTP_FROM");
        }
        assert!(
            !EnvConfig::is_available(),
            "is_available should be false when required env vars are missing"
        );
    }

    #[test]
    fn require_var_returns_value_when_set() {
        // SAFETY: test is single-threaded.
        unsafe {
            std::env::set_var("GITBLAME_TEST_VAR_1234", "hello");
        }
        let val = require_var("GITBLAME_TEST_VAR_1234").unwrap();
        assert_eq!(val, "hello");
        // Clean up.
        unsafe {
            std::env::remove_var("GITBLAME_TEST_VAR_1234");
        }
    }
}
