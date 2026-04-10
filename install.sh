#!/usr/bin/env bash
# ============================================================================
#  git-blame-2.0 installer
#  "From Passive-Aggressive Forensics to Active-Aggressive Email Automation"
#
#  Usage:
#    Install:    curl -fsSL https://gitblame.org/install.sh | bash
#    Uninstall:  curl -fsSL https://gitblame.org/install.sh | bash -s -- --uninstall
# ============================================================================

set -euo pipefail

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
REPO="BarishNamazov/gitblame"
INSTALL_DIR="$HOME/.gitblame"
BIN_DIR="$INSTALL_DIR/bin"
RELEASE_BASE="https://github.com/$REPO/releases/latest/download"

# ---------------------------------------------------------------------------
# Colors & style
# ---------------------------------------------------------------------------
BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
RESET='\033[0m'

info()    { printf "${CYAN}ℹ${RESET}  %s\n" "$*"; }
success() { printf "${GREEN}✔${RESET}  %s\n" "$*"; }
warn()    { printf "${YELLOW}⚠${RESET}  %s\n" "$*"; }
error()   { printf "${RED}✖${RESET}  %s\n" "$*" >&2; }
fatal()   { error "$@"; exit 1; }

# ---------------------------------------------------------------------------
# Uninstall
# ---------------------------------------------------------------------------
uninstall() {
    printf "\n${MAGENTA}🗑  Uninstalling git-blame-2.0…${RESET}\n\n"

    if [ -d "$INSTALL_DIR" ]; then
        rm -rf "$INSTALL_DIR"
        success "Removed $INSTALL_DIR"
    else
        warn "$INSTALL_DIR does not exist — nothing to remove."
    fi

    # Clean shell configs
    local configs=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.config/fish/config.fish")
    for rc in "${configs[@]}"; do
        if [ -f "$rc" ]; then
            # Remove lines added by this installer (identified by marker comment)
            if grep -q "# gitblame-install" "$rc" 2>/dev/null; then
                # Create a cleaned version without our lines
                sed -i.gitblame-backup '/# gitblame-install$/d' "$rc"
                success "Cleaned $rc (backup at ${rc}.gitblame-backup)"
            fi
        fi
    done

    printf "\n${GREEN}${BOLD}✅ git-blame-2.0 has been uninstalled.${RESET}\n"
    printf "   Your real git is unharmed. It was always the favorite child.\n"
    printf "   Restart your shell or open a new terminal.\n\n"
    exit 0
}

# Handle --uninstall flag
if [ "${1:-}" = "--uninstall" ] || [ "${1:-}" = "uninstall" ]; then
    uninstall
fi

# ============================================================================
#  Installation begins
# ============================================================================

printf "\n"
printf "${MAGENTA}${BOLD}  ┌─────────────────────────────────────────────────┐${RESET}\n"
printf "${MAGENTA}${BOLD}  │         git-blame-2.0  —  Installer             │${RESET}\n"
printf "${MAGENTA}${BOLD}  │  Passive-Aggressive Forensics Since 2026 (TM)   │${RESET}\n"
printf "${MAGENTA}${BOLD}  └─────────────────────────────────────────────────┘${RESET}\n"
printf "\n"

# ---------------------------------------------------------------------------
# Detect OS
# ---------------------------------------------------------------------------
detect_os() {
    local uname_s
    uname_s="$(uname -s)"
    case "$uname_s" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        CYGWIN*|MINGW*|MSYS*)
            warn "Windows detected (via $uname_s). Using linux binary for WSL/MSYS."
            echo "linux" ;;
        *)
            fatal "Unsupported OS: $uname_s. We blame whoever wrote your kernel." ;;
    esac
}

# ---------------------------------------------------------------------------
# Detect architecture
# ---------------------------------------------------------------------------
detect_arch() {
    local uname_m
    uname_m="$(uname -m)"
    case "$uname_m" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)   echo "aarch64" ;;
        *)
            fatal "Unsupported architecture: $uname_m. git-blame-2.0 only judges on mainstream platforms." ;;
    esac
}

# ---------------------------------------------------------------------------
# Detect shell config file
# ---------------------------------------------------------------------------
detect_shell_config() {
    local shell_name
    shell_name="$(basename "${SHELL:-/bin/sh}")"

    case "$shell_name" in
        zsh)  echo "$HOME/.zshrc" ;;
        fish) echo "$HOME/.config/fish/config.fish" ;;
        bash) echo "$HOME/.bashrc" ;;
        *)
            warn "Unknown shell '$shell_name'. Falling back to ~/.bashrc"
            echo "$HOME/.bashrc" ;;
    esac
}

# ---------------------------------------------------------------------------
# Find the real git binary (before we shadow it)
# ---------------------------------------------------------------------------
find_real_git() {
    # If GITBLAME_REAL_GIT is already set and valid, use it
    if [ -n "${GITBLAME_REAL_GIT:-}" ] && [ -x "$GITBLAME_REAL_GIT" ]; then
        echo "$GITBLAME_REAL_GIT"
        return
    fi

    # Search PATH, skipping our install directory
    local IFS=':'
    for dir in $PATH; do
        # Skip our own bin directory
        if [ "$dir" = "$BIN_DIR" ]; then
            continue
        fi
        local candidate="$dir/git"
        if [ -x "$candidate" ]; then
            echo "$candidate"
            return
        fi
    done

    fatal "Could not find git on your PATH. Please install git first.\n   (Yes, you need the original git to use git-blame-2.0. It's called 'standing on the shoulders of giants.')"
}

# ---------------------------------------------------------------------------
# Write shell config snippet
# ---------------------------------------------------------------------------
write_shell_config() {
    local config_file="$1"
    local real_git_path="$2"
    local shell_name
    shell_name="$(basename "${SHELL:-/bin/sh}")"

    # Ensure the config file's parent directory exists (for fish)
    mkdir -p "$(dirname "$config_file")"

    # Remove old gitblame lines if present (idempotent)
    if [ -f "$config_file" ] && grep -q "# gitblame-install" "$config_file" 2>/dev/null; then
        sed -i.gitblame-backup '/# gitblame-install$/d' "$config_file"
        info "Cleaned previous gitblame entries from $config_file"
    fi

    if [ "$shell_name" = "fish" ]; then
        {
            echo "set -gx GITBLAME_REAL_GIT \"$real_git_path\" # gitblame-install"
            echo "fish_add_path --prepend \"$BIN_DIR\" # gitblame-install"
        } >> "$config_file"
    else
        {
            echo "export GITBLAME_REAL_GIT=\"$real_git_path\" # gitblame-install"
            echo "export PATH=\"$BIN_DIR:\$PATH\" # gitblame-install"
        } >> "$config_file"
    fi

    success "Updated $config_file"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
    # 1. Detect platform
    local os arch
    os="$(detect_os)"
    arch="$(detect_arch)"
    info "Detected platform: ${BOLD}${os}-${arch}${RESET}"

    # 2. Check for required tools
    for cmd in curl tar; do
        if ! command -v "$cmd" &>/dev/null; then
            fatal "'$cmd' is required but not found. Please install it."
        fi
    done

    # 3. Find the real git before we do anything
    local real_git
    real_git="$(find_real_git)"
    success "Found real git: $real_git"

    # 4. Build download URL
    local tarball="gitblame-${os}-${arch}.tar.gz"
    local url="${RELEASE_BASE}/${tarball}"
    info "Downloading from: $url"

    # 5. Create install directory (detect upgrade)
    local is_upgrade=false
    if [ -x "$BIN_DIR/git" ]; then
        is_upgrade=true
        info "Existing installation found — upgrading in place."
    fi
    mkdir -p "$BIN_DIR"

    # 6. Download and extract
    local download_path="$INSTALL_DIR/$tarball"
    if ! curl -fSL --progress-bar "$url" -o "$download_path"; then
        fatal "Download failed. Check that a release exists for ${os}-${arch}.\n   URL: $url"
    fi
    success "Downloaded $tarball"

    # 7. Extract binary
    tar -xzf "$download_path" -C "$BIN_DIR"
    rm -f "$download_path"

    # Ensure the binary is named 'git' and is executable
    if [ ! -f "$BIN_DIR/git" ]; then
        # Maybe the tarball extracted to a different name
        local extracted
        extracted="$(find "$BIN_DIR" -maxdepth 1 -type f -name 'gitblame*' -o -name 'git' | head -1)"
        if [ -n "$extracted" ] && [ "$extracted" != "$BIN_DIR/git" ]; then
            mv "$extracted" "$BIN_DIR/git"
        fi
    fi

    if [ ! -f "$BIN_DIR/git" ]; then
        fatal "Could not find the git binary after extraction. The tarball may have an unexpected structure."
    fi

    chmod +x "$BIN_DIR/git"
    success "Installed binary to $BIN_DIR/git"

    # 8. Handle macOS Gatekeeper (unsigned binary)
    if [ "$os" = "macos" ]; then
        info "Clearing macOS quarantine flag (unsigned binary)…"
        xattr -d com.apple.quarantine "$BIN_DIR/git" 2>/dev/null || true
    fi

    # 9. Update shell config
    local shell_config
    shell_config="$(detect_shell_config)"
    write_shell_config "$shell_config" "$real_git"

    # 10. Print success message
    printf "\n"
    if [ "$is_upgrade" = true ]; then
        printf "${GREEN}${BOLD}  ┌─────────────────────────────────────────────────┐${RESET}\n"
        printf "${GREEN}${BOLD}  │         ✅  Upgrade complete!                    │${RESET}\n"
        printf "${GREEN}${BOLD}  └─────────────────────────────────────────────────┘${RESET}\n"
    else
        printf "${GREEN}${BOLD}  ┌─────────────────────────────────────────────────┐${RESET}\n"
        printf "${GREEN}${BOLD}  │         ✅  Installation complete!               │${RESET}\n"
        printf "${GREEN}${BOLD}  └─────────────────────────────────────────────────┘${RESET}\n"
    fi
    printf "\n"
    printf "  ${BOLD}Binary:${RESET}          $BIN_DIR/git\n"
    printf "  ${BOLD}Real git:${RESET}        $real_git\n"
    printf "  ${BOLD}Shell config:${RESET}    $shell_config\n"
    printf "\n"
    if [ "$is_upgrade" = true ]; then
        printf "  ${MAGENTA}${BOLD}🎉 git-blame-2.0 has been upgraded.${RESET}\n"
        printf "  ${MAGENTA}   The accountability engine grows stronger.${RESET}\n"
    else
        printf "  ${CYAN}Next steps:${RESET}\n"
        printf "    1. Restart your shell (or run: ${BOLD}source $shell_config${RESET})\n"
        printf "    2. Create a ${BOLD}.env${RESET} file with your SMTP & API credentials\n"
        printf "    3. Optionally create a ${BOLD}.gitblame${RESET} config file\n"
        printf "    4. Run ${BOLD}git blame${RESET} on someone who deserves it\n"
        printf "\n"
        printf "  ${MAGENTA}${BOLD}🎉 Your git is now Sophisticated AI™ powered.${RESET}\n"
        printf "  ${MAGENTA}   Every 'git blame' is now a professional yet devastating email.${RESET}\n"
        printf "  ${MAGENTA}   HR has been notified. (Just kidding. Or are we?)${RESET}\n"
    fi
    printf "\n"
    printf "  ${YELLOW}To uninstall:${RESET} curl -fsSL https://gitblame.org/install.sh | bash -s -- --uninstall\n"
    printf "\n"
}

main "$@"
