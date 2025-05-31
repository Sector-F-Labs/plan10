#!/bin/bash
# Plan 10 CLI One-liner Installer
# curl -sSf https://raw.githubusercontent.com/plan10/plan10/main/install-one-liner.sh | sh

set -e

# Colors
R='\033[0;31m'; G='\033[0;32m'; Y='\033[1;33m'; B='\033[0;34m'; N='\033[0m'

info() { printf "${B}info${N}: %s\n" "$1"; }
warn() { printf "${Y}warn${N}: %s\n" "$1"; }
error() { printf "${R}error${N}: %s\n" "$1" >&2; exit 1; }
success() { printf "${G}success${N}: %s\n" "$1"; }

# Check OS
case "$(uname -s)" in
    Darwin|Linux) info "Detected $(uname -s)" ;;
    *) error "Unsupported OS. Plan 10 CLI supports macOS and Linux only." ;;
esac

# Install Rust if needed
if ! command -v cargo >/dev/null 2>&1; then
    info "Installing Rust..."
    command -v curl >/dev/null 2>&1 || error "curl required but not installed"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    . "$HOME/.cargo/env" 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"
    command -v cargo >/dev/null 2>&1 || error "Rust installation failed"
    success "Rust installed"
else
    info "Rust already installed"
fi

# Install Plan 10 CLI
info "Installing Plan 10 CLI..."
command -v git >/dev/null 2>&1 || error "git required but not installed"
cargo install --git https://github.com/plan10/plan10 --root "$HOME/.cargo" plan10-cli || error "Plan 10 CLI installation failed"
success "Plan 10 CLI installed"

# Check PATH
if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    warn "Add to your shell profile: export PATH=\"\$HOME/.cargo/bin:\$PATH\""
fi

# Test installation
if command -v plan10 >/dev/null 2>&1; then
    success "Installation complete! Run 'plan10 --help' to get started"
else
    warn "Installed but not in PATH. Run: source ~/.cargo/env"
fi

info "Quick start: plan10 setup"