# Installation & Release Guide

> **git-blame-2.0**: From Passive-Aggressive Forensics to Active-Aggressive Email Automation

This document covers how to build, release, install, configure, and uninstall `git-blame-2.0`.

---

## Table of Contents

- [Quick Install](#quick-install)
- [Installation Methods](#installation-methods)
- [Release Process](#release-process)
- [Cross-Platform Notes](#cross-platform-notes)
- [Configuration](#configuration)
- [Uninstallation](#uninstallation)

---

## Quick Install

```bash
curl -fsSL https://gitblame.org/install.sh | bash
```

That's it. Your `git blame` is now *Sophisticated AI™ powered*.

---

## Installation Methods

### 1. One-liner curl install (recommended)

```bash
curl -fsSL https://gitblame.org/install.sh | bash
```

This will:
- Download the correct binary for your OS and architecture
- Install it to `~/.gitblame/bin/git`
- Find your real `git` binary and save its path
- Add `~/.gitblame/bin` to the front of your `PATH`
- Set `GITBLAME_REAL_GIT` in your shell config

### 2. Manual download from releases

1. Go to [Releases](https://github.com/BarishNamazov/gitblame/releases)
2. Download the tarball for your platform:
   - `gitblame-linux-x86_64.tar.gz`
   - `gitblame-linux-aarch64.tar.gz`
   - `gitblame-macos-x86_64.tar.gz` (Intel)
   - `gitblame-macos-aarch64.tar.gz` (Apple Silicon)
3. Extract and install:

```bash
mkdir -p ~/.gitblame/bin
tar -xzf gitblame-<os>-<arch>.tar.gz -C ~/.gitblame/bin
chmod +x ~/.gitblame/bin/git
```

4. Find your real git and configure your shell:

```bash
# Find the real git (usually /usr/bin/git)
which git

# Add to your shell config (~/.bashrc, ~/.zshrc, etc.)
export GITBLAME_REAL_GIT="/usr/bin/git"
export PATH="$HOME/.gitblame/bin:$PATH"
```

### 3. Build from source

```bash
git clone https://github.com/BarishNamazov/gitblame.git
cd gitblame/cli
cargo build --release
```

The binary will be at `cli/target/release/git`. Copy it to `~/.gitblame/bin/git` and configure your shell as described in the manual install above.

---

## Release Process

### Building release binaries

#### Local builds (native platform only)

```bash
cd cli
cargo build --release
# Binary: target/release/git
```

#### Cross-compilation for all targets

Install cross-compilation toolchains:

```bash
# Install Rust targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for each target
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

> **Tip:** For Linux cross-compilation to aarch64, you'll need the `aarch64-linux-gnu-gcc` linker. For macOS targets from Linux, consider using [`cross`](https://github.com/cross-rs/cross) or building on an actual macOS machine.

#### Packaging tarballs

Each tarball should contain a single `git` binary:

```bash
# Naming convention: gitblame-{os}-{arch}.tar.gz
# The binary inside must be named 'git'

# Linux x86_64
cp target/x86_64-unknown-linux-gnu/release/git ./git
tar -czf gitblame-linux-x86_64.tar.gz git
rm git

# Linux aarch64
cp target/aarch64-unknown-linux-gnu/release/git ./git
tar -czf gitblame-linux-aarch64.tar.gz git
rm git

# macOS x86_64 (Intel)
cp target/x86_64-apple-darwin/release/git ./git
tar -czf gitblame-macos-x86_64.tar.gz git
rm git

# macOS aarch64 (Apple Silicon)
cp target/aarch64-apple-darwin/release/git ./git
tar -czf gitblame-macos-aarch64.tar.gz git
rm git
```

### Creating a release

Build the tarballs for every target (see above), then publish them manually with `gh`:

```bash
gh release create v0.1.0 \
  gitblame-linux-x86_64.tar.gz \
  gitblame-linux-aarch64.tar.gz \
  gitblame-macos-x86_64.tar.gz \
  gitblame-macos-aarch64.tar.gz \
  --title "v0.1.0" \
  --generate-notes
```

### Tarball naming convention

| Platform | Tarball name |
|----------|-------------|
| Linux x86_64 | `gitblame-linux-x86_64.tar.gz` |
| Linux aarch64 (ARM64) | `gitblame-linux-aarch64.tar.gz` |
| macOS x86_64 (Intel) | `gitblame-macos-x86_64.tar.gz` |
| macOS aarch64 (Apple Silicon) | `gitblame-macos-aarch64.tar.gz` |

Each tarball contains a single binary named `git`.

---

## Cross-Platform Notes

### Linux

The most straightforward platform. The install script handles everything automatically. Both x86_64 and aarch64 are supported.

### macOS

- **Gatekeeper**: Since the binary is unsigned, macOS will quarantine it. The install script runs `xattr -d com.apple.quarantine` to clear this. If you download manually, you may need to run this yourself or go to **System Settings → Privacy & Security** and click "Allow Anyway".
- **Apple Silicon (M1/M2/M3)**: Use the `macos-aarch64` build. Rosetta 2 can run the `macos-x86_64` build, but the native ARM build is preferred.

### Windows

- **WSL (recommended)**: Use the Linux binary inside WSL. The install script detects WSL and downloads the Linux build.
- **MSYS2 / Git Bash**: The install script also supports MSYS/Cygwin environments and will use the Linux binary.
- **Native Windows**: Not currently supported. If you're using Windows without WSL, we recommend WSL 2. Or reconsider your life choices. (Just kidding. Mostly.)

### PATH ordering (important!)

`git-blame-2.0` works by placing its `git` binary *before* the real git on your `PATH`. This means:

- `~/.gitblame/bin` must come **before** `/usr/bin` (or wherever your real git lives) in `PATH`
- The tool finds the real git by either:
  1. Using the `GITBLAME_REAL_GIT` environment variable (explicit override)
  2. Scanning `PATH` for the next `git` binary that isn't itself

If something goes wrong with PATH resolution, set `GITBLAME_REAL_GIT` explicitly:

```bash
export GITBLAME_REAL_GIT="/usr/bin/git"
```

---

## Configuration

### .env file

Required for email and AI features. Create a `.env` file in your repository root (or point `GITBLAME_DOTENV` at one anywhere on disk):

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

> Without `.env` credentials, the enhanced blame/forgive/therapy commands won't work, but all standard git commands pass through normally.

### .gitblame config file

Optional. Create a `.gitblame` file (TOML format) in your repository root — or point `GITBLAME_DOTGITBLAME` at one anywhere on disk:

```toml
[general]
tone = "passive-aggressive"    # gentle | firm | passive-aggressive | scorched-earth
cc = ["team-lead@org.com"]
cc_group = "eng-all@org.com"   # CC the whole team (for maximum effect)
escalation_threshold = 3       # Blames before auto-escalation
model = "google/gemma-4-31b-it:free"  # OpenRouter model for Sophisticated AI™

[severity]
unused_import = "gentle"
todo_in_prod = "firm"
eval_in_loop = "scorched-earth"
force_push_to_main = "scorched-earth"
```

### Environment overrides

- `GITBLAME_REAL_GIT` — absolute path to the real `git` binary (fallback when PATH detection fails).
- `GITBLAME_DOTENV` — absolute path to an alternate `.env` file.
- `GITBLAME_DOTGITBLAME` — absolute path to an alternate `.gitblame` config file (skips the upward directory search).

```bash
export GITBLAME_REAL_GIT="/usr/bin/git"
export GITBLAME_DOTENV="$HOME/.config/gitblame/.env"
export GITBLAME_DOTGITBLAME="$HOME/.config/gitblame/.gitblame"
```

---

## Uninstallation

### Automatic uninstall

```bash
curl -fsSL https://gitblame.org/install.sh | bash -s -- --uninstall
```

Or if you still have the script locally:

```bash
bash install.sh --uninstall
```

### Manual uninstall

1. Remove the install directory:
   ```bash
   rm -rf ~/.gitblame
   ```

2. Remove these lines from your shell config (`~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish`):
   ```bash
   export GITBLAME_REAL_GIT="/usr/bin/git"  # gitblame-install
   export PATH="$HOME/.gitblame/bin:$PATH"  # gitblame-install
   ```

3. Restart your shell.

That's it. Your git is now back to its boring, non-judgmental self. We won't take it personally.

*(Sophisticated AI™ has noted this in your permanent record.)*
