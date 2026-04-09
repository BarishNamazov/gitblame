# git-blame-2.0

> Peer-Reviewed & Peer-Feared — SIGTBD 2026

**git-blame-2.0** closes the feedback loop Linus Torvalds left open twenty years ago. Sophisticated AI™ identifies the culprit, generates a contextually devastating email, and delivers it before you can say "it worked on my machine."

It's a drop-in `git` shim: every normal command passes through to the real git untouched. The interesting commands are the new ones.

---

## Table of Contents

- [Quick Install](#quick-install)
- [How It Works](#how-it-works)
- [Commands](#commands)
- [Configuration](#configuration)
  - [.env file](#env-file)
  - [.gitblame config](#gitblame-config)
- [Platform Notes](#platform-notes)
- [Uninstall](#uninstall)
- [Building from Source](#building-from-source)
- [Full Install & Release Guide](#full-install--release-guide)
- [License](#license)

---

## Quick Install

```bash
curl -fsSL https://gitblame.org/install.sh | bash
```

That's it. Your `git blame` is now *Sophisticated AI™ powered*.

The installer:
- Downloads the correct binary for your OS/arch
- Installs it to `~/.gitblame/bin/git`
- Detects your real `git` and saves the path to `GITBLAME_REAL_GIT`
- Prepends `~/.gitblame/bin` to your `PATH` in your shell config

Restart your shell (or `source` your rc file) and you're done.

---

## How It Works

`git-blame-2.0` installs a binary literally named `git` into `~/.gitblame/bin`, which is placed **before** the real git on your `PATH`. Every invocation of `git` hits the shim first:

- **Unknown subcommands** (anything that isn't a `git-blame-2.0` feature) are forwarded verbatim to the real git — stdin, stdout, stderr, exit code, all preserved. You won't notice it's there.
- **Recognized subcommands** (`blame`, `forgive`, `therapy`, `we-need-to-talk`, …) are intercepted and routed through the Sophisticated AI™ pipeline.

The real git is found via:
1. The `GITBLAME_REAL_GIT` environment variable (explicit override), or
2. A `PATH` scan for the next `git` that isn't the shim itself.

---

## Commands

All standard git commands work as normal. The extras:

### `git blame <file>`
Enhanced blame. Identifies the culprit, classifies the offense severity, drafts an email, and (depending on tone) either previews it or sends it.

### `git forgive <commit>`
The opposite of blame. Writes a magnanimous email to the author absolving them of the sin. CC's their manager for plausible deniability.

### `git therapy`
AI-mediated conflict resolution between blamer, blamee, and Sophisticated AI™ as neutral mediator. Transcripts are committed to the `healing` branch.

### `git we-need-to-talk [<author>]`
Escalates blame to a 30-minute calendar invite titled "Sync re: Code Quality Concerns." No agenda. No dial-in. Attendance is mandatory.

### `git gud [--user <author>] [--no-u]`
Sends a personalized improvement plan based on commit history analysis. Includes Udemy courses and a mass card from the team. `--no-u` redirects the plan back at the sender (warning: has been known to infinite-loop an SMTP server).

### `git git`
Wellness check. When you type `git` repeatedly without a subcommand, the shim recognizes the distress signal and offers options: walk, coffee, close terminal, career change. No judgment.

> Run any of the email-sending commands with `--dry-run` to preview the generated email without sending.

---

## Configuration

### .env file

Required for AI and email features. Place a `.env` in your repo root (or `$HOME/.gitblame/.env`):

```env
# LLM API key (required for AI-generated emails)
OPENROUTER_API_KEY=your-api-key-here

# SMTP settings (required for sending emails)
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=you@example.com
SMTP_PASSWORD=your-smtp-password
SMTP_FROM=you@example.com
SMTP_ENCRYPTION=starttls   # "starttls" or "tls"
```

Without credentials, the enhanced commands won't work, but pass-through git commands still behave normally.

### .gitblame config

Optional. A TOML file in your repo root:

```toml
[general]
tone = "passive-aggressive"    # gentle | firm | passive-aggressive | scorched-earth
cc = ["team-lead@org.com"]
cc_group = "eng-all@org.com"   # CC the whole team (for maximum effect)
escalation_threshold = 3       # Blames before auto-escalation
model = "google/gemma-4-31b-it:free"  # OpenRouter model for Sophisticated AI™

[severity]
unused-import = "gentle"
todo-in-prod = "firm"
eval-in-loop = "scorched-earth"
force-push-to-main = "instant-termination"
```

### Environment overrides

- `GITBLAME_REAL_GIT` — absolute path to the real `git` binary (use this if automatic detection fails).
- `GITBLAME_DOTENV` — absolute path to an alternate `.env` file. When set, it's loaded instead of the default lookup.
- `GITBLAME_DOTGITBLAME` — absolute path to an alternate `.gitblame` config file. When set, it's used directly and the upward directory walk is skipped.

```bash
export GITBLAME_REAL_GIT="/usr/bin/git"
export GITBLAME_DOTENV="$HOME/.config/gitblame/.env"
export GITBLAME_DOTGITBLAME="$HOME/.config/gitblame/.gitblame"
```

---

## Platform Notes

**Linux** — Works out of the box on x86_64 and aarch64.

**macOS** — Intel and Apple Silicon both supported. The binary is unsigned, so Gatekeeper will quarantine it. The installer runs `xattr -d com.apple.quarantine` for you; if you install manually you may need to click "Allow Anyway" in **System Settings → Privacy & Security**. We're not paying Apple $99/year to sign a tool that sends passive-aggressive emails.

**Windows** — Use WSL. The installer detects WSL/MSYS/Cygwin and pulls the Linux binary. Native Windows is not supported.

**PATH ordering matters.** `~/.gitblame/bin` must come **before** `/usr/bin` (or wherever your real git lives). The installer handles this; if you edit your shell config by hand, keep the order intact.

---

## Uninstall

```bash
curl -fsSL https://gitblame.org/install.sh | bash -s -- --uninstall
```

Or manually:

1. `rm -rf ~/.gitblame`
2. Remove the two `# gitblame-install` lines from your shell rc file.
3. Restart your shell.

Your git returns to its boring, non-judgmental self. We won't take it personally. *(Sophisticated AI™ has noted this in your permanent record.)*

---

## Building from Source

```bash
git clone https://github.com/BarishNamazov/gitblame.git
cd gitblame/cli
cargo build --release
```

The binary lands at `cli/target/release/git`. Copy it to `~/.gitblame/bin/git` and configure your shell as described in the install guide.

Run the test suite:

```bash
cd cli
cargo test
```

---

## Full Install & Release Guide

For cross-compilation, CI release workflow, tarball naming conventions, and manual install paths, see [INSTALL.md](INSTALL.md).

---

## License

See [LICENSE](LICENSE) if present, otherwise assume all blame is reserved.
