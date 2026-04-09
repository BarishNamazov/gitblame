//! Enhanced `git blame` — identifies the culprit and sends an AI-generated
//! blame email with full context.

use crate::ai::AiClient;
use crate::config::BlameConfig;
use crate::email::{BlameEmail, EmailClient};
use crate::env_config::EnvConfig;
use crate::error::Result;
use crate::git::{get_blame_info, get_file_context, get_user_commits};

/// Run the enhanced blame workflow for a file (and optional line range).
///
/// 1. Runs `git blame` to identify the author.
/// 2. Gathers surrounding file context and recent commit history.
/// 3. Asks the AI to draft a blame email using the configured tone.
/// 4. Sends the email to the blamee (CC list from `.gitblame`).
pub fn execute(
    file: &str,
    line: Option<&str>,
    config: &BlameConfig,
    env: &EnvConfig,
) -> Result<()> {
    // -- 1. Blame ---------------------------------------------------------
    eprintln!("🔍 Analyzing blame for {file}...");
    let blame = get_blame_info(file, line)?;

    println!("Blamed: {} <{}>", blame.author, blame.email);
    println!("Commit: {} ({})", blame.commit_hash, blame.date);
    println!("Line:   {}", blame.line_content.trim());

    // -- 2. Context -------------------------------------------------------
    let line_no: usize = line
        .and_then(|l| l.split(',').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let context = get_file_context(file, line_no, 5).unwrap_or_default();

    // -- 3. Commit history ------------------------------------------------
    eprintln!("📜 Fetching recent commit history for {}...", blame.author);
    let commits = get_user_commits(&blame.author, 10)?;
    let history = commits
        .iter()
        .map(|c| format!("  {} {} — {}", c.hash.get(..8).unwrap_or(&c.hash), c.date, c.message))
        .collect::<Vec<_>>()
        .join("\n");

    // -- 4. AI prompt -----------------------------------------------------
    eprintln!("🤖 Generating blame email (tone: {})...", config.general.tone);
    let prompt = format!(
        "Write a blame email with the following details.\n\n\
         Tone: {tone}\n\n\
         Blamed code:\n```\n{context}```\n\n\
         Author: {author} <{email}>\n\
         Commit: {hash}\n\
         Date: {date}\n\
         Commit message: {msg}\n\n\
         Recent commit history for this author:\n{history}\n\n\
         Instructions:\n\
         - Address the email to the author by first name.\n\
         - Reference the specific line of code.\n\
         - Comment on patterns visible in the commit history if relevant.\n\
         - Match the requested tone precisely.\n\
         - Include a subject line on the first line prefixed with \"Subject: \".\n\
         - Keep it under 300 words.\n\
         - Sign off as 'Sophisticated AI™'.",
        tone = config.general.tone,
        context = context,
        author = blame.author,
        email = blame.email,
        hash = blame.commit_hash,
        date = blame.date,
        msg = blame.commit_message,
        history = if history.is_empty() {
            "  (no recent commits found)".to_string()
        } else {
            history
        },
    );

    let ai = AiClient::new(&env.openrouter_api_key, &config.general.model);
    let response = ai.generate(&prompt)?;

    // Parse subject from AI response (first line starting with "Subject: ").
    let (subject, body) = parse_subject_body(&response);

    println!("\n--- Generated Email ---");
    println!("Subject: {subject}");
    println!("{body}");
    println!("--- End ---\n");

    // -- 5. Send ----------------------------------------------------------
    eprintln!("📧 Sending blame email to {}...", blame.email);
    let mut cc = config.general.cc.clone();
    if let Some(ref group) = config.general.cc_group {
        cc.push(group.clone());
    }

    let email_client = EmailClient::new(env)?;
    email_client.send_blame_email(&BlameEmail {
        to: blame.email.clone(),
        cc,
        subject,
        body,
    })?;

    eprintln!("✅ Blame email sent to {}. Justice has been served.", blame.email);
    Ok(())
}

/// Split the AI response into a subject line and body.
pub(crate) fn parse_subject_body(response: &str) -> (String, String) {
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
    // Fallback: use a generic subject.
    (
        "Code Quality Concern".to_string(),
        response.trim().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_subject_body_with_subject_line() {
        let response = "Subject: Your Code Needs Attention\n\nDear Alice,\n\nPlease review.";
        let (subject, body) = parse_subject_body(response);
        assert_eq!(subject, "Your Code Needs Attention");
        assert_eq!(body, "Dear Alice,\n\nPlease review.");
    }

    #[test]
    fn parse_subject_body_no_subject_line() {
        let response = "Just a plain response with no subject prefix.";
        let (subject, body) = parse_subject_body(response);
        assert_eq!(subject, "Code Quality Concern");
        assert_eq!(body, "Just a plain response with no subject prefix.");
    }

    #[test]
    fn parse_subject_body_subject_not_on_first_line() {
        let response = "Hello,\nSubject: Late Subject\nBody text here.";
        let (subject, body) = parse_subject_body(response);
        assert_eq!(subject, "Late Subject");
        assert_eq!(body, "Body text here.");
    }

    #[test]
    fn parse_subject_body_empty_input() {
        let response = "";
        let (subject, body) = parse_subject_body(response);
        assert_eq!(subject, "Code Quality Concern");
        assert_eq!(body, "");
    }

    #[test]
    fn parse_subject_body_trims_whitespace() {
        let response = "Subject:   Spaces Everywhere   \n\n  Body with spaces  \n";
        let (subject, body) = parse_subject_body(response);
        assert_eq!(subject, "Spaces Everywhere");
        assert!(body.contains("Body with spaces"));
    }
}
