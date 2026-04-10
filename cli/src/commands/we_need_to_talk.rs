//! `git we-need-to-talk` — sends a calendar-invite-style email for a
//! 30-minute "Code Quality Concerns" meeting.

use crate::ai::AiClient;
use crate::config::BlameConfig;
use crate::email::{BlameEmail, EmailClient};
use crate::env_config::EnvConfig;
use crate::error::Result;
use crate::git::run_git;

/// Send a dreaded calendar-invite email to the specified user.
///
/// The email contains an ICS-style meeting block in the body:
/// - Title: "Sync re: Code Quality Concerns"
/// - Duration: 30 minutes
/// - Description: "You know what this is about."
pub fn execute(user: &str, config: &BlameConfig, env: &EnvConfig) -> Result<()> {
    eprintln!("📅 Scheduling a talk with {user}...");

    let email = lookup_email(user)?;
    eprintln!("📬 Found email for {user}: {email}");

    // -- Generate the meeting email via AI --------------------------------
    eprintln!("🤖 Generating meeting invite email...");

    let ics_block = build_ics_block(&env.smtp_from, &email);

    let prompt = format!(
        "Write a short, ominous email inviting someone to a meeting.\n\n\
         Tone: {tone}\n\
         Recipient name: {user}\n\
         Meeting title: Sync re: Code Quality Concerns\n\
         Duration: 30 minutes\n\
         Description: You know what this is about.\n\n\
         Instructions:\n\
         - The email should feel like receiving a calendar invite from your manager \
           on a Friday afternoon.\n\
         - Do NOT include the ICS data — that will be appended automatically.\n\
         - Include a subject line on the first line prefixed with \"Subject: \".\n\
         - Keep the email body under 150 words.\n\
         - Match the requested tone.\n\
         - Sign off as 'Sophisticated AI™' with 'https://gitblame.org' on the next line.",
        tone = config.general.tone,
        user = user,
    );

    let ai = AiClient::new(&env.openrouter_api_key, &config.general.model);
    let response = ai.generate(&prompt)?;

    let (subject, ai_body) = parse_subject_body(&response);
    let full_body = format!(
        "{ai_body}\n\n\
         ──────────────────────────────────\n\
         📎 Calendar Invite (ICS)\n\
         ──────────────────────────────────\n\n\
         {ics_block}"
    );

    println!("\n--- Meeting Invite Email ---");
    println!("Subject: {subject}");
    println!("{full_body}");
    println!("--- End ---\n");

    // -- Send -------------------------------------------------------------
    eprintln!("📧 Sending meeting invite to {email}...");
    let mut cc = config.general.cc.clone();
    if let Some(ref group) = config.general.cc_group {
        cc.push(group.clone());
    }

    let email_client = EmailClient::new(env)?;
    let blame_email = BlameEmail {
        to: email.clone(),
        cc,
        subject,
        body: full_body,
    }
    .apply_demo_override(env.demo_email_address.as_deref());
    email_client.send_blame_email(&blame_email)?;

    eprintln!("✅ Meeting invite sent to {email}. They'll know what it's about.");
    Ok(())
}

/// Build a minimal ICS calendar event block (as plain text for the body).
fn build_ics_block(organizer: &str, attendee: &str) -> String {
    // Use a deterministic "tomorrow at 2 PM UTC" placeholder so the text is
    // reproducible.  A real implementation would pick an actual slot.
    format!(
        "BEGIN:VCALENDAR\n\
         VERSION:2.0\n\
         PRODID:-//git-blame-2.0//EN\n\
         BEGIN:VEVENT\n\
         SUMMARY:Sync re: Code Quality Concerns\n\
         DESCRIPTION:You know what this is about.\n\
         DTSTART:20260101T140000Z\n\
         DTEND:20260101T143000Z\n\
         ORGANIZER:mailto:{organizer}\n\
         ATTENDEE:mailto:{attendee}\n\
         STATUS:CONFIRMED\n\
         END:VEVENT\n\
         END:VCALENDAR"
    )
}

/// Resolve a user name to an email from `git log --author`.
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
        "Sync re: Code Quality Concerns".to_string(),
        response.trim().to_string(),
    )
}
