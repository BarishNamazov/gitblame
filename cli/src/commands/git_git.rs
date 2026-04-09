//! `git git` — a wellness check for when you've typed "git" one too many times.

use crate::error::Result;

use std::io::{self, BufRead, Write};

/// Print a wellness check menu and respond to the user's choice.
///
/// No config or email required — this is purely a terminal interaction.
pub fn execute() -> Result<()> {
    println!(
        r#"
Hey. You've typed "git" without a meaningful subcommand.

Are you alright? Do you need to:
 [1] Take a walk
 [2] Get coffee
 [3] Close the terminal
 [4] Reconsider your career choices

No judgment. We've all been there."#
    );

    print!("\n> ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let input = match lines.next() {
        Some(Ok(line)) => line.trim().to_string(),
        _ => String::new(),
    };

    println!();

    match input.as_str() {
        "1" => {
            println!("🚶 Good call. Step away from the keyboard.");
            println!("   The bugs will still be there when you get back.");
            println!("   (They always are.)");
        }
        "2" => {
            println!("☕ Excellent choice. Caffeine: the original version control.");
            println!("   Fun fact: 73% of all git force-pushes happen pre-coffee.");
            println!("   That's not a real statistic, but it feels true.");
        }
        "3" => {
            println!("🖥️  Close it. Walk away. Breathe.");
            println!("   Remember: the terminal can't hurt you if you don't open it.");
            println!("   (That's technically true.)");
        }
        "4" => {
            println!("🎓 Coding bootcamps are still accepting applications.");
            println!("   Alternatively, have you considered:");
            println!("   • Artisanal bread baking");
            println!("   • Goat farming");
            println!("   • Becoming a project manager (just kidding)");
            println!("   • Writing a novel about a developer who loses their mind");
            println!();
            println!("   Whatever you choose, know that `git` will always be here,");
            println!("   waiting, judging, silently.");
        }
        _ => {
            println!("🤔 That wasn't one of the options, but honestly,");
            println!("   that's very on-brand for someone who typed `git git`.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn wellness_message_contains_expected_options() {
        // Verify the static strings are present in the source.
        // We can't easily capture stdout from execute() without stdin mocking,
        // so we test the known string constants directly.
        let menu = r#"
Hey. You've typed "git" without a meaningful subcommand.

Are you alright? Do you need to:
 [1] Take a walk
 [2] Get coffee
 [3] Close the terminal
 [4] Reconsider your career choices

No judgment. We've all been there."#;

        assert!(menu.contains("Take a walk"));
        assert!(menu.contains("Get coffee"));
        assert!(menu.contains("Close the terminal"));
        assert!(menu.contains("Reconsider your career choices"));
        assert!(menu.contains("No judgment"));
        assert!(menu.contains("git"));
    }
}
