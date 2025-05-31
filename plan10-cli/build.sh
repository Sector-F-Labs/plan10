#!/bin/bash

# Plan 10 CLI Build Script
# Builds the Rust CLI for Plan 10 MacBook server management

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    local rust_version=$(rustc --version)
    print_status "Found Rust: $rust_version"
}

# Check if we're in the right directory
check_directory() {
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Cargo.toml not found. Please run this script from the plan10-cli directory."
        exit 1
    fi
    
    if [[ ! -f "src/main.rs" ]]; then
        print_error "src/main.rs not found. Please ensure the source code is present."
        exit 1
    fi
    
    print_status "Source directory verified"
}

# Clean previous builds
clean_build() {
    if [[ -d "target" ]]; then
        print_status "Cleaning previous build artifacts..."
        cargo clean
        print_success "Build artifacts cleaned"
    fi
}

# Update dependencies
update_deps() {
    print_status "Updating dependencies..."
    cargo update
    print_success "Dependencies updated"
}

# Run tests
run_tests() {
    print_status "Running tests..."
    if cargo test; then
        print_success "All tests passed"
    else
        print_error "Tests failed"
        exit 1
    fi
}

# Build release binary
build_release() {
    print_status "Building release binary..."
    
    export RUSTFLAGS="-C target-cpu=native"
    
    if cargo build --release; then
        print_success "Release binary built successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Check binary
check_binary() {
    local binary_path="target/release/plan10"
    
    if [[ -f "$binary_path" ]]; then
        local binary_size=$(ls -lh "$binary_path" | awk '{print $5}')
        print_success "Binary created: $binary_path ($binary_size)"
        
        # Test binary
        if "$binary_path" --version &> /dev/null; then
            local version=$("$binary_path" --version)
            print_success "Binary test successful: $version"
        else
            print_warning "Binary test failed - binary may not be functional"
        fi
    else
        print_error "Binary not found at $binary_path"
        exit 1
    fi
}

# Install binary
install_binary() {
    local binary_path="target/release/plan10"
    local install_path="/usr/local/bin/plan10"
    
    if [[ "$1" == "--install" ]]; then
        print_status "Installing binary to $install_path..."
        
        if command -v sudo &> /dev/null; then
            sudo cp "$binary_path" "$install_path"
            sudo chmod +x "$install_path"
            print_success "Binary installed to $install_path"
        else
            print_warning "sudo not available. Please manually copy $binary_path to your PATH"
        fi
    else
        print_status "To install the binary, run: $0 --install"
        print_status "Or manually copy: cp $binary_path /usr/local/bin/"
    fi
}

# Create distribution package
create_package() {
    if [[ "$1" == "--package" ]]; then
        print_status "Creating distribution package..."
        
        local version=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
        local package_name="plan10-cli-v${version}-$(uname -s)-$(uname -m)"
        local package_dir="dist/$package_name"
        
        mkdir -p "$package_dir"
        
        # Copy binary
        cp target/release/plan10 "$package_dir/"
        
        # Copy documentation
        cp README.md "$package_dir/"
        cp ../LICENSE "$package_dir/"
        
        # Create install script
        cat > "$package_dir/install.sh" << 'EOF'
#!/bin/bash
echo "Installing Plan 10 CLI..."
sudo cp plan10 /usr/local/bin/
sudo chmod +x /usr/local/bin/plan10
echo "Plan 10 CLI installed successfully!"
echo "Run 'plan10 --help' to get started."
EOF
        chmod +x "$package_dir/install.sh"
        
        # Create archive
        cd dist
        tar -czf "$package_name.tar.gz" "$package_name"
        cd ..
        
        print_success "Package created: dist/$package_name.tar.gz"
    fi
}

# Show usage
show_usage() {
    echo "Plan 10 CLI Build Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --help      Show this help message"
    echo "  --clean     Clean build artifacts before building"
    echo "  --no-tests  Skip running tests"
    echo "  --install   Install binary to /usr/local/bin after building"
    echo "  --package   Create distribution package"
    echo "  --dev       Build development version (faster compile)"
    echo ""
    echo "Examples:"
    echo "  $0                    # Standard build"
    echo "  $0 --clean --install # Clean build and install"
    echo "  $0 --package         # Build and create package"
    echo "  $0 --dev             # Quick development build"
}

# Parse command line arguments
CLEAN=false
NO_TESTS=false
INSTALL=false
PACKAGE=false
DEV=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --help)
            show_usage
            exit 0
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --no-tests)
            NO_TESTS=true
            shift
            ;;
        --install)
            INSTALL=true
            shift
            ;;
        --package)
            PACKAGE=true
            shift
            ;;
        --dev)
            DEV=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main build process
main() {
    print_status "Starting Plan 10 CLI build process..."
    
    check_rust
    check_directory
    
    if [[ "$CLEAN" == true ]]; then
        clean_build
    fi
    
    update_deps
    
    if [[ "$NO_TESTS" != true ]]; then
        run_tests
    fi
    
    if [[ "$DEV" == true ]]; then
        print_status "Building development binary..."
        cargo build
        print_success "Development binary built: target/debug/plan10"
    else
        build_release
        check_binary
        
        if [[ "$INSTALL" == true ]]; then
            install_binary --install
        else
            install_binary
        fi
        
        if [[ "$PACKAGE" == true ]]; then
            create_package --package
        fi
    fi
    
    print_success "Build process completed successfully!"
}

# Run main function
main "$@"