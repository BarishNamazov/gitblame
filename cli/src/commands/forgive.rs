//! `git forgive` — sends an AI-generated retraction / forgiveness email.

use crate::ai::AiClient;
use crate::config::BlameConfig;
use crate::email::{BlameEmail, EmailClient};
use crate::env_config::EnvConfig;
use crate::error::Result;
use crate::git::run_git;

/// Look up a user's email from `git log` and send a forgiveness email.
///
/// The retraction is CC'd to the original CC list so everyone knows
/// the grudge has been officially dropped.
pub fn execute(user: &str, config: &BlameConfig, env: &EnvConfig) -> Result<()> {
    eprintln!("🕊️  Preparing forgiveness for {user}...");

    // -- Resolve email from git log --------------------------------------
    let email = lookup_email(user)?;
    eprintln!("📬 Found email for {user}: {email}");

    // -- Generate forgiveness email via AI --------------------------------
    eprintln!("🤖 Generating forgiveness email (tone: {})...", config.general.tone);
    let prompt = format!(
        "Write a retraction / forgiveness email.\n\n\
         Tone: {tone}\n\
         Recipient name: {user}\n\
         Recipient email: {email}\n\n\
         Instructions:\n\
         - This is a follow-up to a previous blame email.\n\
         - Officially retract the blame and express forgiveness.\n\
         - Make it clear the recipient is no longer under scrutiny.\n\
         - Match the requested tone (e.g. if passive-aggressive, \
           forgive in a way that still stings a little).\n\
         - Include a subject line on the first line prefixed with \"Subject: \".\n\
         - CC recipients will be notified automatically — mention that \
           the team has been informed of the retraction.\n\
         - Keep it under 200 words.\n\
         - Sign off as 'Sophisticated AI™'.",
        tone = config.general.tone,
        user = user,
        email = email,
    );

    let ai = AiClient::new(&env.openrouter_api_key, &config.general.model);
    let response = ai.generate(&prompt)?;

    let (subject, body) = parse_subject_body(&response);

    println!("\n--- Forgiveness Email ---");
    println!("Subject: {subject}");
    println!("{body}");
    println!("--- End ---\n");

    // -- Send -------------------------------------------------------------
    eprintln!("📧 Sending forgiveness email to {email}...");
    let mut cc = config.general.cc.clone();
    if let Some(ref group) = config.general.cc_group {
        cc.push(group.clone());
    }

    let email_client = EmailClient::new(env)?;
    email_client.send_blame_email(&BlameEmail {
        to: email.clone(),
        cc,
        subject,
        body,
    })?;

    eprintln!("✅ Forgiveness sent to {email}. The record has been expunged.");
    Ok(())
}

/// Resolve a user name or email from `git log --author`.
fn lookup_email(user: &str) -> Result<String> {
    let output = run_git(&[
        "log",
        "--all",
        "--format=%ae",
        &format!("--author={user}"),
        "-1",
    ])?;
    let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if email.is_empty() {
        return Err(format!(
            "Could not find an email for \"{user}\" in git history. \
             Make sure the name matches a commit author."
        )
        .into());
    }
    Ok(email)
}

/// Split the AI response into a subject line and body.
fn parse_subject_body(response: &str) -> (String, String) {
    for (i, line) in response.lines().enumerate() {
        if let Some(subj) = line.strip_prefix("Subject: ") {
            let body = response
                .lines()
                .skip(i + 1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            return (subj.trim().to_string(), body);
        }
    }
    (
        "Official Retraction of Blame".to_string(),
        response.trim().to_string(),
    )
}
