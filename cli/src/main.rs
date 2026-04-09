pub mod ai;
pub mod commands;
pub mod config;
pub mod email;
pub mod env_config;
pub mod error;
pub mod git;

use std::process;

/// git-blame-2.0: From Passive-Aggressive Forensics to Active-Aggressive Email
/// Automation.
///
/// This binary acts as a transparent proxy to the real `git`, but intercepts
/// certain subcommands (`blame`, `forgive`, `we-need-to-talk`, `therapy`,
/// `gud`, `git`) and extends them with Sophisticated AI™ and automated email
/// delivery.
///
/// If no `.gitblame` config file is present and no `.env` credentials are
/// available, all standard git commands pass through unchanged.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // No arguments → just run real git (shows help).
    if args.is_empty() {
        passthrough_and_exit(&[]);
    }

    let subcommand = args[0].as_str();
    let rest: Vec<&str> = args[1..].iter().map(String::as_str).collect();

    match subcommand {
        // -- Enhanced blame --------------------------------------------------
        "blame" => {
            // If there are flags that look like plain git blame usage
            // (e.g. --help, --porcelain, etc.) just pass through.
            if rest.iter().any(|a| a.starts_with("--") || a.starts_with("-p")) {
                passthrough_and_exit(&args_as_str(&args));
            }

            // We need at least a file path to do the enhanced blame.
            // Parse: git blame <file> [-L <line>]
            let (file, line) = parse_blame_args(&rest);

            match file {
                Some(f) => run_enhanced_blame(&f, line.as_deref()),
                None => passthrough_and_exit(&args_as_str(&args)),
            }
        }

        // -- git forgive <user> ----------------------------------------------
        "forgive" => {
            let user = rest.first().copied().unwrap_or_else(|| {
                eprintln!("Usage: git forgive <user>");
                process::exit(1);
            });
            run_with_config(|cfg, env| commands::forgive::execute(user, cfg, env));
        }

        // -- git we-need-to-talk <user> --------------------------------------
        "we-need-to-talk" => {
            let user = rest.first().copied().unwrap_or_else(|| {
                eprintln!("Usage: git we-need-to-talk <user>");
                process::exit(1);
            });
            run_with_config(|cfg, env| commands::we_need_to_talk::execute(user, cfg, env));
        }

        // -- git therapy <user> ----------------------------------------------
        "therapy" => {
            let user = rest.first().copied().unwrap_or_else(|| {
                eprintln!("Usage: git therapy <user>");
                process::exit(1);
            });
            run_with_config(|cfg, env| commands::therapy::execute(user, cfg, env));
        }

        // -- git gud [--user <user>] [--no-u] --------------------------------
        "gud" => {
            let (user, no_u) = parse_gud_args(&rest);
            let user = user.unwrap_or_else(|| {
                eprintln!("Usage: git gud --user <user> [--no-u]");
                process::exit(1);
            });
            run_with_config(|cfg, env| commands::gud::execute(&user, no_u, cfg, env));
        }

        // -- git git (wellness check) ----------------------------------------
        "git" => {
            if let Err(e) = commands::git_git::execute() {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }

        // -- Everything else: pass through to real git -----------------------
        _ => passthrough_and_exit(&args_as_str(&args)),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse blame arguments: `git blame <file> [-L <line>]`
fn parse_blame_args(rest: &[&str]) -> (Option<String>, Option<String>) {
    let mut file = None;
    let mut line = None;
    let mut i = 0;

    while i < rest.len() {
        let arg = rest[i];
        if arg == "-L" {
            if i + 1 < rest.len() {
                line = Some(rest[i + 1].to_string());
                i += 2;
                continue;
            }
        } else if let Some(l) = arg.strip_prefix("-L") {
            line = Some(l.to_string());
            i += 1;
            continue;
        } else if !arg.starts_with('-') {
            file = Some(arg.to_string());
        }
        i += 1;
    }

    (file, line)
}

/// Parse `git gud` arguments: `--user <name>` and `--no-u`.
fn parse_gud_args(rest: &[&str]) -> (Option<String>, bool) {
    let mut user = None;
    let mut no_u = false;
    let mut i = 0;

    while i < rest.len() {
        match rest[i] {
            "--user" => {
                if i + 1 < rest.len() {
                    user = Some(rest[i + 1].to_string());
                    i += 2;
                    continue;
                }
            }
            "--no-u" => no_u = true,
            other if !other.starts_with('-') && user.is_none() => {
                user = Some(other.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    (user, no_u)
}

/// Run an enhanced blame command.
fn run_enhanced_blame(file: &str, line: Option<&str>) {
    let config = config::BlameConfig::load();
    match env_config::EnvConfig::load() {
        Ok(env) => {
            if let Err(e) = commands::blame::execute(file, line, &config, &env) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
        Err(_) => {
            // No env config → fall back to standard git blame.
            let mut args = vec!["blame"];
            if let Some(l) = line {
                args.push("-L");
                // We need to leak the string to get a &str with the right lifetime.
                // This is fine — we're about to exit anyway.
                args.push(Box::leak(l.to_string().into_boxed_str()));
            }
            args.push(Box::leak(file.to_string().into_boxed_str()));
            passthrough_and_exit(&args);
        }
    }
}

/// Load config + env and run a command that needs both.
fn run_with_config<F>(f: F)
where
    F: FnOnce(&config::BlameConfig, &env_config::EnvConfig) -> error::Result<()>,
{
    let config = config::BlameConfig::load();
    let env = match env_config::EnvConfig::load() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("❌ git-blame-2.0 requires configuration to send emails.");
            eprintln!("   {e}");
            eprintln!();
            eprintln!("   Create a .env file with:");
            eprintln!("     OPENROUTER_API_KEY=<your key>");
            eprintln!("     SMTP_HOST=<smtp host>");
            eprintln!("     SMTP_USERNAME=<username>");
            eprintln!("     SMTP_PASSWORD=<password>");
            eprintln!("     SMTP_FROM=<from address>");
            process::exit(1);
        }
    };

    if let Err(e) = f(&config, &env) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

/// Pass arguments through to the real git and exit with its status code.
fn passthrough_and_exit(args: &[&str]) -> ! {
    match git::run_git_passthrough(args) {
        Ok(status) => process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("git-blame-2.0: failed to execute git: {e}");
            process::exit(127);
        }
    }
}

/// Convert a `Vec<String>` to a `Vec<&str>`.
fn args_as_str(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_blame_args tests --

    #[test]
    fn parse_blame_args_file_only() {
        let args = vec!["src/main.rs"];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, Some("src/main.rs".to_string()));
        assert_eq!(line, None);
    }

    #[test]
    fn parse_blame_args_file_and_line_separate() {
        let args = vec!["src/main.rs", "-L", "42"];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, Some("src/main.rs".to_string()));
        assert_eq!(line, Some("42".to_string()));
    }

    #[test]
    fn parse_blame_args_file_and_line_combined() {
        let args = vec!["src/main.rs", "-L42"];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, Some("src/main.rs".to_string()));
        assert_eq!(line, Some("42".to_string()));
    }

    #[test]
    fn parse_blame_args_line_before_file() {
        let args = vec!["-L", "10", "README.md"];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, Some("README.md".to_string()));
        assert_eq!(line, Some("10".to_string()));
    }

    #[test]
    fn parse_blame_args_empty() {
        let args: Vec<&str> = vec![];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, None);
        assert_eq!(line, None);
    }

    #[test]
    fn parse_blame_args_line_range() {
        let args = vec!["file.rs", "-L", "10,20"];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, Some("file.rs".to_string()));
        assert_eq!(line, Some("10,20".to_string()));
    }

    #[test]
    fn parse_blame_args_dangling_l_flag() {
        // -L at end with no value — should not set line
        let args = vec!["file.rs", "-L"];
        let (file, line) = parse_blame_args(&args);
        assert_eq!(file, Some("file.rs".to_string()));
        assert_eq!(line, None);
    }

    // -- parse_gud_args tests --

    #[test]
    fn parse_gud_args_user_flag() {
        let args = vec!["--user", "alice"];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, Some("alice".to_string()));
        assert!(!no_u);
    }

    #[test]
    fn parse_gud_args_positional_user() {
        let args = vec!["bob"];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, Some("bob".to_string()));
        assert!(!no_u);
    }

    #[test]
    fn parse_gud_args_user_with_no_u() {
        let args = vec!["--user", "charlie", "--no-u"];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, Some("charlie".to_string()));
        assert!(no_u);
    }

    #[test]
    fn parse_gud_args_no_u_before_user() {
        let args = vec!["--no-u", "--user", "dave"];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, Some("dave".to_string()));
        assert!(no_u);
    }

    #[test]
    fn parse_gud_args_empty() {
        let args: Vec<&str> = vec![];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, None);
        assert!(!no_u);
    }

    #[test]
    fn parse_gud_args_only_no_u() {
        let args = vec!["--no-u"];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, None);
        assert!(no_u);
    }

    #[test]
    fn parse_gud_args_dangling_user_flag() {
        // --user at end with no value
        let args = vec!["--user"];
        let (user, no_u) = parse_gud_args(&args);
        assert_eq!(user, None);
        assert!(!no_u);
    }

    #[test]
    fn parse_gud_args_positional_not_overridden_by_flag() {
        // Positional user is set first, --user should override
        let args = vec!["alice", "--user", "bob"];
        let (user, no_u) = parse_gud_args(&args);
        // "alice" gets set as positional, then --user sets "bob"
        assert_eq!(user, Some("bob".to_string()));
        assert!(!no_u);
    }
}
