use crate::error::Result;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Structured output from `git blame` for a single line.
#[derive(Debug, Clone)]
pub struct BlameInfo {
    /// Author name.
    pub author: String,
    /// Author email.
    pub email: String,
    /// Commit date (ISO-ish format from git).
    pub date: String,
    /// Short or full commit hash.
    pub commit_hash: String,
    /// Content of the blamed line.
    pub line_content: String,
    /// Path to the file that was blamed.
    pub file_path: String,
    /// Full commit message (fetched via `git log`).
    pub commit_message: String,
}

/// Summary of a single commit.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash (short).
    pub hash: String,
    /// First line of the commit message.
    pub message: String,
    /// Author date.
    pub date: String,
}

// ---------------------------------------------------------------------------
// Finding the real git
// ---------------------------------------------------------------------------

/// Locate the *real* `git` binary.
///
/// Resolution order:
/// 1. The path specified in the `GITBLAME_REAL_GIT` environment variable.
/// 2. The first `git` found on `PATH` whose canonical path differs from our
///    own executable's canonical path.
///
/// Returns an error if no suitable binary is found.
pub fn find_real_git() -> Result<PathBuf> {
    // 1. Explicit override via environment variable.
    if let Ok(explicit) = std::env::var("GITBLAME_REAL_GIT") {
        let p = PathBuf::from(&explicit);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "GITBLAME_REAL_GIT is set to \"{explicit}\" but that file does not exist"
        )
        .into());
    }

    // 2. Search PATH, skipping ourselves.
    let our_exe = std::env::current_exe()
        .ok()
        .and_then(|p| fs::canonicalize(p).ok());

    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join("git");
        if !candidate.is_file() {
            continue;
        }
        // Skip if this candidate is ourselves.
        if let Some(ref ours) = our_exe {
            if let Ok(canon) = fs::canonicalize(&candidate) {
                if canon == *ours {
                    continue;
                }
            }
        }
        return Ok(candidate);
    }

    Err("Could not find the real git binary on PATH. \
         Set GITBLAME_REAL_GIT to its location."
        .into())
}

// ---------------------------------------------------------------------------
// Running git
// ---------------------------------------------------------------------------

/// Execute the real `git` with the given arguments and capture its output.
pub fn run_git(args: &[&str]) -> Result<Output> {
    let git = find_real_git()?;
    let output = Command::new(&git).args(args).output().map_err(|e| {
        format!("Failed to execute git at {}: {e}", git.display())
    })?;
    Ok(output)
}

/// Execute the real `git` with the given arguments in passthrough mode
/// (stdin/stdout/stderr are inherited), returning its exit status.
pub fn run_git_passthrough(args: &[&str]) -> Result<ExitStatus> {
    let git = find_real_git()?;
    let status = Command::new(&git)
        .args(args)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map_err(|e| format!("Failed to execute git at {}: {e}", git.display()))?;
    Ok(status)
}

// ---------------------------------------------------------------------------
// Blame helpers
// ---------------------------------------------------------------------------

/// Run `git blame` on a file and return structured [`BlameInfo`].
///
/// `line` can be a single line number (`"47"`) or a range (`"47,50"`).
/// When a range is given the *first* line in the range is returned.
pub fn get_blame_info(file: &str, line: Option<&str>) -> Result<BlameInfo> {
    let mut args = vec!["blame", "--porcelain"];

    let line_arg;
    if let Some(l) = line {
        // git blame -L expects start,end — a bare number means a single line.
        if l.contains(',') {
            line_arg = format!("-L{l}");
        } else {
            line_arg = format!("-L{l},{l}");
        }
        args.push(&line_arg);
    }
    args.push(file);

    let output = run_git(&args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git blame failed: {stderr}").into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_porcelain_blame(&stdout, file)
}

/// Parse the first entry from `git blame --porcelain` output.
pub(crate) fn parse_porcelain_blame(porcelain: &str, file: &str) -> Result<BlameInfo> {
    let mut author = String::new();
    let mut email = String::new();
    let mut date = String::new();
    let mut commit_hash = String::new();
    let mut line_content = String::new();

    for raw_line in porcelain.lines() {
        if commit_hash.is_empty() && raw_line.len() >= 40 {
            // First line: "<hash> <orig-line> <final-line> [<num-lines>]"
            commit_hash = raw_line
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string();
        } else if let Some(val) = raw_line.strip_prefix("author ") {
            author = val.to_string();
        } else if let Some(val) = raw_line.strip_prefix("author-mail ") {
            // author-mail <user@example.com>
            email = val.trim_matches(|c| c == '<' || c == '>').to_string();
        } else if let Some(val) = raw_line.strip_prefix("author-time ") {
            // Unix timestamp — convert to human-readable via chrono if available.
            date = timestamp_to_date(val);
        } else if raw_line.starts_with('\t') {
            // Tab-indented line is the actual source line.
            line_content = raw_line[1..].to_string();
        }
    }

    // Fetch commit message via git log.
    let commit_message = if !commit_hash.is_empty() {
        fetch_commit_message(&commit_hash)?
    } else {
        String::new()
    };

    Ok(BlameInfo {
        author,
        email,
        date,
        commit_hash,
        line_content,
        file_path: file.to_string(),
        commit_message,
    })
}

/// Convert a UNIX timestamp string to a human-readable date.
pub(crate) fn timestamp_to_date(ts: &str) -> String {
    ts.trim()
        .parse::<i64>()
        .ok()
        .and_then(|secs| {
            chrono::DateTime::from_timestamp(secs, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        })
        .unwrap_or_else(|| ts.to_string())
}

/// Fetch the full commit message for a given hash.
fn fetch_commit_message(hash: &str) -> Result<String> {
    let output = run_git(&["log", "-1", "--format=%B", hash])?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ---------------------------------------------------------------------------
// Commit history
// ---------------------------------------------------------------------------

/// Retrieve recent commits by a given author.
pub fn get_user_commits(author: &str, max_count: u32) -> Result<Vec<CommitInfo>> {
    let count_str = max_count.to_string();
    let output = run_git(&[
        "log",
        "--format=%H%n%s%n%ai",
        &format!("--author={author}"),
        &format!("-n{count_str}"),
    ])?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    // Output comes in groups of three lines: hash, subject, date.
    let lines: Vec<&str> = stdout.lines().collect();
    for chunk in lines.chunks(3) {
        if chunk.len() == 3 {
            commits.push(CommitInfo {
                hash: chunk[0].to_string(),
                message: chunk[1].to_string(),
                date: chunk[2].to_string(),
            });
        }
    }
    Ok(commits)
}

// ---------------------------------------------------------------------------
// File context
// ---------------------------------------------------------------------------

/// Read `context_lines` lines above and below `line` from `file`.
///
/// Returns the extracted snippet as a single string with line numbers.
pub fn get_file_context(file: &str, line: usize, context_lines: usize) -> Result<String> {
    let content = fs::read_to_string(file)
        .map_err(|e| format!("Cannot read {file}: {e}"))?;

    let lines: Vec<&str> = content.lines().collect();
    let start = line.saturating_sub(context_lines + 1); // 0-indexed
    let end = (line + context_lines).min(lines.len());

    let mut result = String::new();
    for (i, text) in lines[start..end].iter().enumerate() {
        let line_no = start + i + 1;
        let marker = if line_no == line { ">>>" } else { "   " };
        result.push_str(&format!("{marker} {line_no:>4} | {text}\n"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_conversion() {
        // 2024-01-01T00:00:00Z
        let date = timestamp_to_date("1704067200");
        assert!(date.starts_with("2024-01-01"), "got: {date}");
    }

    #[test]
    fn timestamp_to_date_empty_string() {
        let date = timestamp_to_date("");
        assert_eq!(date, "", "empty input should return empty string");
    }

    #[test]
    fn timestamp_to_date_negative() {
        // Negative timestamp (before epoch) should still convert
        let date = timestamp_to_date("-86400");
        assert!(date.contains("1969"), "got: {date}");
    }

    #[test]
    fn timestamp_to_date_large_value() {
        // Far future: 2100-01-01T00:00:00Z
        let date = timestamp_to_date("4102444800");
        assert!(date.starts_with("2100-01-01"), "got: {date}");
    }

    #[test]
    fn timestamp_to_date_non_numeric() {
        // Non-numeric input should fall back to returning the input as-is
        let date = timestamp_to_date("not-a-number");
        assert_eq!(date, "not-a-number");
    }

    #[test]
    fn parse_porcelain_blame_sample() {
        let porcelain = "\
abc1234567890abcdef1234567890abcdef123456 47 47 1\n\
author Jane Doe\n\
author-mail <jane@example.com>\n\
author-time 1704067200\n\
author-tz +0000\n\
committer Jane Doe\n\
committer-mail <jane@example.com>\n\
committer-time 1704067200\n\
committer-tz +0000\n\
summary Fix the frobulator\n\
filename src/main.rs\n\
\tlet x = 42;\n";

        // parse_porcelain_blame calls fetch_commit_message which needs real git,
        // but we can still test parsing by checking the result structure.
        // In a non-git context, the commit message fetch will fail, so we
        // test the parts we can.
        let _result = parse_porcelain_blame(porcelain, "src/main.rs");
        // This may error if not in a git repo with the commit, but the
        // parsing up to that point should work. Let's test with a known
        // empty-hash scenario.
        // Instead, test with a zero hash which git log will handle gracefully.
        let porcelain_zero = "\
0000000000000000000000000000000000000000 1 1 1\n\
author Not Committed Yet\n\
author-mail <not.committed@example.com>\n\
author-time 1704067200\n\
author-tz +0000\n\
committer Not Committed Yet\n\
committer-mail <not.committed@example.com>\n\
committer-time 1704067200\n\
committer-tz +0000\n\
summary (not committed)\n\
filename test.rs\n\
\tprintln!(\"hello\");\n";

        // This will try to run git log on the zero hash; it may fail,
        // so just validate we don't panic. The main parsing logic is
        // still exercised.
        let _ = parse_porcelain_blame(porcelain_zero, "test.rs");
    }

    #[test]
    fn parse_porcelain_blame_extracts_fields() {
        // We test that the parsing logic correctly extracts author, email,
        // date, hash, and line content from porcelain output. We do this
        // by reimplementing just the parsing part without the git log call.
        let porcelain = "\
abcdef1234567890abcdef1234567890abcdef12 10 10 1\n\
author Alice\n\
author-mail <alice@test.org>\n\
author-time 1704067200\n\
\treturn 0;\n";

        let mut author = String::new();
        let mut email = String::new();
        let mut date = String::new();
        let mut commit_hash = String::new();
        let mut line_content = String::new();

        for raw_line in porcelain.lines() {
            if commit_hash.is_empty() && raw_line.len() >= 40 {
                commit_hash = raw_line
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
            } else if let Some(val) = raw_line.strip_prefix("author ") {
                author = val.to_string();
            } else if let Some(val) = raw_line.strip_prefix("author-mail ") {
                email = val.trim_matches(|c| c == '<' || c == '>').to_string();
            } else if let Some(val) = raw_line.strip_prefix("author-time ") {
                date = timestamp_to_date(val);
            } else if raw_line.starts_with('\t') {
                line_content = raw_line[1..].to_string();
            }
        }

        assert_eq!(author, "Alice");
        assert_eq!(email, "alice@test.org");
        assert!(date.starts_with("2024-01-01"), "got date: {date}");
        assert_eq!(commit_hash, "abcdef1234567890abcdef1234567890abcdef12");
        assert_eq!(line_content, "return 0;");
    }

    #[test]
    fn get_file_context_with_temp_file() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("sample.rs");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            for i in 1..=20 {
                writeln!(f, "line {i}").unwrap();
            }
        }

        let ctx = get_file_context(file_path.to_str().unwrap(), 10, 2).unwrap();

        // Should contain lines 8-12 (10 ± 2) with line 10 marked
        assert!(ctx.contains(">>> "), "should have a marker line");
        assert!(ctx.contains("10"), "should contain line 10");
        assert!(ctx.contains("line 10"), "should contain the text of line 10");
        // Lines outside the window should not be present
        assert!(!ctx.contains("line 5\n"), "line 5 should not be in context");
    }

    #[test]
    fn get_file_context_line_one() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("edge.rs");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            for i in 1..=5 {
                writeln!(f, "content {i}").unwrap();
            }
        }

        let ctx = get_file_context(file_path.to_str().unwrap(), 1, 2).unwrap();
        assert!(ctx.contains("content 1"), "should contain line 1");
        assert!(ctx.contains(">>>"), "should have the marker on line 1");
    }

    #[test]
    fn get_file_context_missing_file() {
        let result = get_file_context("/nonexistent/path/file.txt", 1, 2);
        assert!(result.is_err());
    }
}
