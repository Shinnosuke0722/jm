#!/bin/sh
# jm installer — https://github.com/lfming0419/jm
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/lfming0419/jm/main/install.sh | sh
#   curl -fsSL https://raw.githubusercontent.com/lfming0419/jm/main/install.sh | sh -s -- --version v0.2.0
#   curl -fsSL https://raw.githubusercontent.com/lfming0419/jm/main/install.sh | sh -s -- --install-dir /usr/local/bin

set -eu

# ─── Configuration ──────────────────────────────────────────────────────────

REPO="lfming0419/jm"
BINARY_NAME="jm"
DEFAULT_INSTALL_DIR="${HOME}/.jm/bin"

# ─── Argument parsing ──────────────────────────────────────────────────────

VERSION=""
INSTALL_DIR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --version)     VERSION="$2";      shift 2 ;;
        --install-dir) INSTALL_DIR="$2";   shift 2 ;;
        --help)
            echo "Usage: install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version VERSION      Install a specific version (e.g. v0.1.0)"
            echo "  --install-dir DIR      Install to a custom directory"
            echo "  --help                 Show this help"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"

# ─── Helpers ────────────────────────────────────────────────────────────────

info() {
    printf '  \033[1;34m>\033[0m %s\n' "$@"
}

error() {
    printf '  \033[1;31merror:\033[0m %s\n' "$@" >&2
    exit 1
}

success() {
    printf '  \033[1;32m>\033[0m %s\n' "$@"
}

# ─── Platform detection ────────────────────────────────────────────────────

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "Unsupported operating system: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

# ─── Downloader detection ──────────────────────────────────────────────────

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

download() {
    url="$1"
    output="$2"

    if has_cmd curl; then
        curl -fsSL "$url" -o "$output"
    elif has_cmd wget; then
        wget -qO "$output" "$url"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

download_text() {
    url="$1"

    if has_cmd curl; then
        curl -fsSL "$url"
    elif has_cmd wget; then
        wget -qO- "$url"
    fi
}

# ─── Version resolution ────────────────────────────────────────────────────

get_latest_version() {
    # Use GitHub API to get the latest release tag
    release_info=$(download_text "https://api.github.com/repos/${REPO}/releases/latest") || \
        error "Failed to fetch latest release info. Check your network or set --version manually."

    # Extract tag_name without requiring jq
    echo "$release_info" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1
}

# ─── Main ───────────────────────────────────────────────────────────────────

main() {
    echo ""
    echo "  \033[1mjm installer\033[0m"
    echo ""

    OS=$(detect_os)
    ARCH=$(detect_arch)
    info "Detected platform: ${OS} ${ARCH}"

    # Resolve version
    if [ -z "$VERSION" ]; then
        info "Fetching latest version..."
        VERSION=$(get_latest_version)
    fi

    if [ -z "$VERSION" ]; then
        error "Could not determine version to install."
    fi
    info "Installing version: ${VERSION}"

    # Determine artifact name (must match release.yml naming)
    case "$OS" in
        macos)
            # Prefer universal binary on macOS
            ARTIFACT="jm-macos-universal"
            EXT="tar.gz"
            ;;
        linux)
            ARTIFACT="jm-linux-${ARCH}"
            EXT="tar.gz"
            ;;
        windows)
            ARTIFACT="jm-windows-${ARCH}"
            EXT="zip"
            ;;
    esac

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}.${EXT}"
    info "Downloading from: ${DOWNLOAD_URL}"

    # Download to a temporary directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    ARCHIVE_PATH="${TMP_DIR}/${ARTIFACT}.${EXT}"
    download "$DOWNLOAD_URL" "$ARCHIVE_PATH" || \
        error "Download failed. Check that version ${VERSION} exists and has release assets."

    # Verify checksum if sha256sum/shasum is available
    CHECKSUMS_URL="https://github.com/${REPO}/releases/download/${VERSION}/sha256sums.txt"
    if has_cmd sha256sum || has_cmd shasum; then
        info "Verifying checksum..."
        CHECKSUMS_FILE="${TMP_DIR}/sha256sums.txt"
        if download "$CHECKSUMS_URL" "$CHECKSUMS_FILE" 2>/dev/null; then
            EXPECTED=$(grep "${ARTIFACT}.${EXT}" "$CHECKSUMS_FILE" | awk '{ print $1 }')
            if [ -n "$EXPECTED" ]; then
                if has_cmd sha256sum; then
                    ACTUAL=$(sha256sum "$ARCHIVE_PATH" | awk '{ print $1 }')
                else
                    ACTUAL=$(shasum -a 256 "$ARCHIVE_PATH" | awk '{ print $1 }')
                fi
                if [ "$EXPECTED" != "$ACTUAL" ]; then
                    error "Checksum mismatch!\n  Expected: ${EXPECTED}\n  Actual:   ${ACTUAL}"
                fi
                success "Checksum verified"
            fi
        fi
    fi

    # Extract
    info "Extracting..."
    case "$EXT" in
        tar.gz) tar xzf "$ARCHIVE_PATH" -C "$TMP_DIR" ;;
        zip)    unzip -qo "$ARCHIVE_PATH" -d "$TMP_DIR" ;;
    esac

    # Install
    mkdir -p "$INSTALL_DIR"
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    success "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"

    # PATH setup guidance
    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
        echo ""
        info "Add jm to your PATH by adding this to your shell config:"
        echo ""

        SHELL_NAME=$(basename "${SHELL:-/bin/sh}")
        case "$SHELL_NAME" in
            bash)
                echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.bashrc"
                echo "    source ~/.bashrc"
                ;;
            zsh)
                echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ~/.zshrc"
                echo "    source ~/.zshrc"
                ;;
            fish)
                echo "    echo 'fish_add_path ${INSTALL_DIR}' >> ~/.config/fish/config.fish"
                echo "    source ~/.config/fish/config.fish"
                ;;
            *)
                echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
                ;;
        esac
    fi

    # Shell integration hint
    echo ""
    info "Then set up shell integration for auto-switching:"
    echo ""
    case "$(basename "${SHELL:-/bin/sh}")" in
        bash) echo "    eval \"\$(jm shell init bash)\"   # add to ~/.bashrc" ;;
        zsh)  echo "    eval \"\$(jm shell init zsh)\"    # add to ~/.zshrc" ;;
        fish) echo "    jm shell init fish | source      # add to ~/.config/fish/config.fish" ;;
        *)    echo "    eval \"\$(jm shell init bash)\"" ;;
    esac

    echo ""
    success "Done! Run 'jm --help' to get started."
    echo ""
}

main
