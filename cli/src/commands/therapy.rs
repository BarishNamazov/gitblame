//! `git therapy` — interactive AI mediator session in the terminal.

use crate::ai::AiClient;
use crate::config::BlameConfig;
use crate::env_config::EnvConfig;
use crate::error::Result;

use std::io::{self, BufRead, Write};

/// Start an interactive therapy / mediation session for the given user.
///
/// The AI acts as a mediator, guiding the developer through their feelings
/// about code quality, deadlines, and coworkers.  The conversation is
/// printed to stdout as a running transcript.
pub fn execute(user: &str, config: &BlameConfig, env: &EnvConfig) -> Result<()> {
    eprintln!("🛋️  Starting therapy session for {user}...\n");

    let ai = AiClient::new(&env.openrouter_api_key, &config.general.model);
    let mut transcript = Vec::<String>::new();

    // Opening message from the mediator.
    let opening_prompt = format!(
        "You are an AI mediator / therapist for software developers.\n\
         Your patient today is {user}.\n\
         Tone: {tone}\n\n\
         Start the session with a warm (but slightly judgmental, matching the tone) \
         opening statement. Ask them what's been bothering them about the codebase. \
         Keep it to 2-3 sentences.",
        user = user,
        tone = config.general.tone,
    );

    let opening = ai.generate(&opening_prompt)?;
    println!("🤖 Mediator: {opening}\n");
    transcript.push(format!("Mediator: {opening}"));

    // Interactive loop.
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut lines = reader.lines();

    loop {
        print!("💬 {user}: ");
        io::stdout().flush()?;

        let input = match lines.next() {
            Some(Ok(line)) => line,
            _ => break,
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.eq_ignore_ascii_case("quit")
            || trimmed.eq_ignore_ascii_case("exit")
            || trimmed.eq_ignore_ascii_case("bye")
        {
            break;
        }

        transcript.push(format!("{user}: {trimmed}"));

        // Build context for the AI from the recent transcript.
        let recent: String = transcript
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let reply_prompt = format!(
            "You are an AI mediator / therapist for software developers.\n\
             Tone: {tone}\n\
             Patient: {user}\n\n\
             Conversation so far:\n{recent}\n\n\
             Respond as the mediator. Guide the conversation constructively \
             (while staying in character for the tone). Keep it to 2-4 sentences.",
            tone = config.general.tone,
            user = user,
            recent = recent,
        );

        let reply = ai.generate(&reply_prompt)?;
        println!("\n🤖 Mediator: {reply}\n");
        transcript.push(format!("Mediator: {reply}"));
    }

    // Session wrap-up.
    println!("\n──────────────────────────────────");
    println!("📝 Session Transcript");
    println!("──────────────────────────────────");
    for line in &transcript {
        println!("  {line}");
    }
    println!("──────────────────────────────────\n");

    eprintln!("✅ Therapy session complete. Remember: it's not your fault (probably).");
    Ok(())
}
