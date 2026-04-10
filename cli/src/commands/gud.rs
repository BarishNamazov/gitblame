//! `git gud` — analyzes commit history and sends a personalized improvement
//! plan email.

use crate::ai::AiClient;
use crate::config::BlameConfig;
use crate::email::{BlameEmail, EmailClient};
use crate::env_config::EnvConfig;
use crate::error::Result;
use crate::git::{get_user_commits, run_git};

/// Analyze the user's commit history and send an AI-generated improvement plan.
///
/// When `no_u` is `true`, the analysis targets the current git user (self)
/// instead of the specified user — the classic "no u" reversal.
pub fn execute(
    user: &str,
    no_u: bool,
    config: &BlameConfig,
    env: &EnvConfig,
) -> Result<()> {
    let target = if no_u {
        eprintln!("🔄 --no-u flag detected. Turning the mirror on yourself...");
        current_git_user()?
    } else {
        user.to_string()
    };

    eprintln!("📊 Analyzing commit history for {target}...");

    let email_addr = lookup_email(&target)?;
    let commits = get_user_commits(&target, 20)?;

    if commits.is_empty() {
        return Err(format!(
            "No commits found for \"{target}\". \
             Either they haven't contributed yet or the name doesn't match."
        )
        .into());
    }

    let history = commits
        .iter()
        .map(|c| format!("  {} {} — {}", c.hash.get(..8).unwrap_or(&c.hash), c.date, c.message))
        .collect::<Vec<_>>()
        .join("\n");

    // -- AI analysis ------------------------------------------------------
    eprintln!("🤖 Generating personalized improvement plan...");
    let prompt = format!(
        "Analyze the following git commit history and write a personalized \
         improvement plan email.\n\n\
         Tone: {tone}\n\
         Developer: {target} <{email}>\n\n\
         Recent commits:\n{history}\n\n\
         Instructions:\n\
         - Identify patterns: bad variable naming, vague commit messages, \
           rushed fixes, repeated mistakes, etc.\n\
         - Be specific — reference actual commit messages from the list.\n\
         - Provide actionable recommendations (at least 3).\n\
         - Include a subject line on the first line prefixed with \"Subject: \".\n\
         - Match the requested tone.\n\
         - Keep it under 400 words.\n\
         - Sign off as 'Sophisticated AI™' with 'https://gitblame.org' on the next line.",
        tone = config.general.tone,
        target = target,
        email = email_addr,
        history = history,
    );

    let ai = AiClient::new(&env.openrouter_api_key, &config.general.model);
    let response = ai.generate(&prompt)?;

    let (subject, body) = parse_subject_body(&response);

    println!("\n--- Improvement Plan Email ---");
    println!("Subject: {subject}");
    println!("{body}");
    println!("--- End ---\n");

    // -- Send -------------------------------------------------------------
    eprintln!("📧 Sending improvement plan to {email_addr}...");
    let mut cc = config.general.cc.clone();
    if let Some(ref group) = config.general.cc_group {
        cc.push(group.clone());
    }

    let email_client = EmailClient::new(env)?;
    let email = BlameEmail {
        to: email_addr.clone(),
        cc,
        subject,
        body,
    }
    .apply_demo_override(env.demo_email_address.as_deref());
    email_client.send_blame_email(&email)?;

    eprintln!("✅ Improvement plan sent to {email_addr}. Growth is a journey.");
    Ok(())
}

/// Get the current git user name from `git config user.name`.
fn current_git_user() -> Result<String> {
    let output = run_git(&["config", "user.name"])?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() {
        return Err(
            "Could not determine your git user.name. Run `git config user.name` to set it."
                .into(),
        );
    }
    Ok(name)
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
        "Your Personalized Improvement Plan".to_string(),
        response.trim().to_string(),
    )
}
