#!/bin/bash
# Plan 10 CLI Installer
# Usage: curl -sSf https://raw.githubusercontent.com/plan10/plan10/main/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/sector-f-labs/plan10"
CLI_PACKAGE="plan10-cli"
RUSTUP_URL="https://sh.rustup.rs"

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

# Helper functions
info() {
    printf "${BLUE}info${NC}: %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${NC}: %s\n" "$1"
}

error() {
    printf "${RED}error${NC}: %s\n" "$1" >&2
    exit 1
}

success() {
    printf "${GREEN}success${NC}: %s\n" "$1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if we're on a supported platform
check_platform() {
    case "$OS" in
        Darwin)
            info "Detected macOS"
            ;;
        Linux)
            info "Detected Linux"
            ;;
        *)
            error "Unsupported operating system: $OS. Plan 10 CLI supports macOS and Linux."
            ;;
    esac
}

# Install Rust and Cargo if not present
install_rust() {
    if command_exists cargo; then
        info "Rust and Cargo are already installed"
        cargo --version
        return 0
    fi

    info "Rust not found. Installing Rust via rustup..."

    if ! command_exists curl; then
        error "curl is required but not installed. Please install curl and try again."
    fi

    # Download and run rustup installer
    curl --proto '=https' --tlsv1.2 -sSf "$RUSTUP_URL" | sh -s -- -y --default-toolchain stable

    # Source the cargo environment
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        . "$HOME/.cargo/env"
    else
        export PATH="$HOME/.cargo/bin:$PATH"
    fi

    # Verify installation
    if command_exists cargo; then
        success "Rust installed successfully"
        cargo --version
    else
        error "Failed to install Rust. Please install manually from https://rustup.rs/"
    fi
}

# Install Plan 10 CLI
install_plan10_cli() {
    info "Installing Plan 10 CLI..."

    # Check if git is available for cargo install from git
    if ! command_exists git; then
        error "git is required but not installed. Please install git and try again."
    fi

    # Install from GitHub repository
    if cargo install --git "$REPO_URL" --root "$HOME/.cargo" "$CLI_PACKAGE"; then
        success "Plan 10 CLI installed successfully"
    else
        error "Failed to install Plan 10 CLI"
    fi
}

# Check if Plan 10 CLI is in PATH
check_path() {
    CARGO_BIN="$HOME/.cargo/bin"

    if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
        warn "Cargo bin directory is not in your PATH"
        info "Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
        echo ""
        info "Or run this command to add it to your current session:"
        echo "export PATH=\"\$HOME/.cargo/bin:\$PATH\""
        echo ""
    fi

    # Test if plan10 command works
    if command_exists plan10; then
        success "Plan 10 CLI is ready to use!"
        info "Run 'plan10 --help' to get started"
    else
        warn "Plan 10 CLI was installed but is not in your current PATH"
        info "You may need to start a new terminal session or run:"
        echo "source ~/.cargo/env"
    fi
}

# Show post-install instructions
show_instructions() {
    echo ""
    info "=== Plan 10 CLI Installation Complete ==="
    echo ""
    info "Quick start:"
    echo "  plan10 setup              # Run interactive setup"
    echo "  plan10 status              # Check system status"
    echo "  plan10 monitor battery     # Monitor battery"
    echo "  plan10 --help              # Show all commands"
    echo ""
    info "Configuration file will be created at:"
    case "$OS" in
        Darwin)
            echo "  ~/Library/Application Support/plan10/config.toml"
            ;;
        Linux)
            echo "  ~/.config/plan10/config.toml"
            ;;
    esac
    echo ""
    info "For more information, visit: $REPO_URL"
}

# Main installation flow
main() {
    info "Installing Plan 10 CLI..."
    echo ""

    # Check platform compatibility
    check_platform

    # Install Rust if needed
    install_rust

    # Install Plan 10 CLI
    install_plan10_cli

    # Check PATH and provide guidance
    check_path

    # Show post-install instructions
    show_instructions
}

# Handle interruption
trap 'error "Installation interrupted"' INT

# Run main function
main "$@"
