#!/bin/bash

# Build script for Codebook VS Code Extension

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    print_error "npm is not installed. Please install Node.js and npm."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    print_error "package.json not found. Please run this script from the vscode-extension directory."
    exit 1
fi

# Parse command line arguments
COMMAND=${1:-build}
SKIP_INSTALL=${2:-false}

case $COMMAND in
    install)
        print_info "Installing dependencies..."
        npm install
        print_success "Dependencies installed successfully!"
        ;;

    build)
        if [ "$SKIP_INSTALL" != "skip-install" ]; then
            print_info "Installing dependencies..."
            npm install
        fi

        print_info "Compiling TypeScript..."
        npm run compile
        print_success "Build completed successfully!"
        ;;

    watch)
        print_info "Starting TypeScript compiler in watch mode..."
        npm run watch
        ;;

    test)
        if [ "$SKIP_INSTALL" != "skip-install" ]; then
            print_info "Installing dependencies..."
            npm install
        fi

        print_info "Running tests..."
        npm test
        print_success "Tests completed!"
        ;;

    lint)
        print_info "Running linter..."
        npm run lint
        print_success "Linting completed!"
        ;;

    package)
        if [ "$SKIP_INSTALL" != "skip-install" ]; then
            print_info "Installing dependencies..."
            npm install
        fi

        print_info "Checking for vsce..."
        if ! command -v vsce &> /dev/null; then
            print_warning "vsce not found. Installing globally..."
            npm install -g @vscode/vsce
        fi

        print_info "Compiling TypeScript..."
        npm run compile

        print_info "Packaging extension..."
        vsce package

        # Find the generated .vsix file
        VSIX_FILE=$(ls -t *.vsix | head -n 1)
        if [ -n "$VSIX_FILE" ]; then
            print_success "Extension packaged successfully: $VSIX_FILE"
            print_info "To install the extension, run:"
            echo "    code --install-extension $VSIX_FILE"
        else
            print_error "Failed to create .vsix package"
            exit 1
        fi
        ;;

    publish)
        print_warning "Publishing to VS Code Marketplace..."
        print_info "Make sure you have a Personal Access Token from https://dev.azure.com"

        if [ "$SKIP_INSTALL" != "skip-install" ]; then
            print_info "Installing dependencies..."
            npm install
        fi

        print_info "Checking for vsce..."
        if ! command -v vsce &> /dev/null; then
            print_warning "vsce not found. Installing globally..."
            npm install -g @vscode/vsce
        fi

        print_info "Compiling TypeScript..."
        npm run compile

        print_info "Publishing extension..."
        vsce publish

        print_success "Extension published successfully!"
        ;;

    clean)
        print_info "Cleaning build artifacts..."
        rm -rf out/
        rm -rf node_modules/
        rm -f *.vsix
        rm -f package-lock.json
        print_success "Clean completed!"
        ;;

    check-server)
        print_info "Checking for codebook-lsp..."

        # Check if codebook-lsp is in PATH
        if command -v codebook-lsp &> /dev/null; then
            SERVER_PATH=$(which codebook-lsp)
            print_success "Found codebook-lsp at: $SERVER_PATH"
            print_info "Version: $(codebook-lsp --version 2>/dev/null || echo 'unknown')"
        else
            print_warning "codebook-lsp not found in PATH"

            # Check common installation locations
            CARGO_BIN="$HOME/.cargo/bin/codebook-lsp"
            if [ -f "$CARGO_BIN" ]; then
                print_info "Found codebook-lsp in cargo bin: $CARGO_BIN"
                print_info "Version: $($CARGO_BIN --version 2>/dev/null || echo 'unknown')"
            else
                print_error "codebook-lsp is not installed"
                print_info "To install codebook-lsp, run:"
                echo "    cargo install codebook-lsp"
                exit 1
            fi
        fi
        ;;

    dev)
        print_info "Starting development environment..."
        print_info "This will compile the extension and open VS Code in extension development mode"

        if [ "$SKIP_INSTALL" != "skip-install" ]; then
            print_info "Installing dependencies..."
            npm install
        fi

        print_info "Compiling TypeScript..."
        npm run compile

        print_info "Opening VS Code in extension development mode..."
        code --extensionDevelopmentPath="$(pwd)" .

        print_info "Starting TypeScript compiler in watch mode..."
        npm run watch
        ;;

    help|--help|-h)
        echo "Codebook VS Code Extension Build Script"
        echo ""
        echo "Usage: ./build.sh [command] [options]"
        echo ""
        echo "Commands:"
        echo "  install       - Install npm dependencies"
        echo "  build         - Build the extension (default)"
        echo "  watch         - Start TypeScript compiler in watch mode"
        echo "  test          - Run tests"
        echo "  lint          - Run linter"
        echo "  package       - Package the extension as .vsix"
        echo "  publish       - Publish extension to VS Code Marketplace"
        echo "  clean         - Clean build artifacts"
        echo "  check-server  - Check if codebook-lsp is installed"
        echo "  dev           - Start development environment"
        echo "  help          - Show this help message"
        echo ""
        echo "Options:"
        echo "  skip-install  - Skip npm install step (use as second argument)"
        echo ""
        echo "Examples:"
        echo "  ./build.sh                    # Build the extension"
        echo "  ./build.sh build skip-install # Build without installing dependencies"
        echo "  ./build.sh package            # Create .vsix package"
        echo "  ./build.sh dev                # Start development environment"
        ;;

    *)
        print_error "Unknown command: $COMMAND"
        echo "Run './build.sh help' for usage information"
        exit 1
        ;;
esac
