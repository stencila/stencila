#!/usr/bin/env bash
# Install the Stencila CLI on macOS or Linux (x86_64 / arm64)

set -uo pipefail
IFS=$'\n\t'

REPO="stencila/stencila"
BIN_NAME="stencila"
DEFAULT_LINUX_DIR="$HOME/.local/bin"
DEFAULT_MAC_DIR="/usr/local/bin"

###############################################################################
# Helper functions
###############################################################################
usage() {
  cat <<EOF
Usage: $(basename "$0") [version] [install-dir]

  version      Git tag to install (default: latest)
  install-dir  Directory to place the '${BIN_NAME}' binary
               (default: ${DEFAULT_MAC_DIR} on macOS, ${DEFAULT_LINUX_DIR} on Linux)

Examples
  Install latest        : $(basename "$0")
  Install v2.0.0        : $(basename "$0") v2.0.0
  Install to custom dir : $(basename "$0") latest /opt/bin
EOF
  exit 0
}

# Print error message and exit
error() {
  echo "❌ $1" >&2
  exit 1
}

# Print warning message
warn() {
  echo "⚠️ $1" >&2
}

# Print informational message
info() {
  echo "ℹ️  $1"
}

# Print success message
success() {
  echo "✅ $1"
}

# Check if command exists
check_command() {
  command -v "$1" >/dev/null 2>&1
}

# Validate version format
validate_version() {
  if [[ "$1" != "latest" && ! "$1" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    warn "Version format '$1' looks unusual (expected format: v1.2.3)"
    read -r -p "Continue anyway? [y/N] " response
    [[ "${response,,}" =~ ^(yes|y)$ ]] || exit 1
  fi
}

###############################################################################
# Check for required tools
###############################################################################
check_dependencies() {
  local missing=0
  for cmd in curl tar; do
    if ! check_command "$cmd"; then
      error "Required command '$cmd' not found. Please install it and try again."
      missing=1
    fi
  done
  [[ $missing -eq 1 ]] && exit 1
}

###############################################################################
# Parse args
###############################################################################
[[ "${1:-}" =~ ^-h|--help$ ]] && usage
check_dependencies

VERSION="${1:-latest}"
INSTALL_DIR="${2:-}"

# Validate version if not latest
[[ "$VERSION" != "latest" ]] && validate_version "$VERSION"

###############################################################################
# Detect OS / Arch ➜ target triple
###############################################################################
OS=$(uname -s)
ARCH=$(uname -m)
case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) error "Unsupported Linux architecture: $ARCH" ;;
    esac
    INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_LINUX_DIR}"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64)  TARGET="aarch64-apple-darwin" ;;
      *) error "Unsupported macOS architecture: $ARCH" ;;
    esac
    INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_MAC_DIR}"
    ;;
  *)
    error "Unsupported OS: $OS"
    ;;
esac

info "Installing for ${OS} (${ARCH}) as ${TARGET}"

###############################################################################
# Verify/create installation directory
###############################################################################
if [[ ! -d "$INSTALL_DIR" ]]; then
  info "Installation directory '$INSTALL_DIR' does not exist, creating it..."
  if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    info "Creating directory requires elevated privileges (sudo)..."
    sudo mkdir -p "$INSTALL_DIR" || error "Failed to create directory '$INSTALL_DIR'"
  fi
fi

# Check if writable
if [[ ! -w "$INSTALL_DIR" ]]; then
  info "Note: You'll need sudo permissions to install to '$INSTALL_DIR'"
fi

###############################################################################
# Resolve "latest" tag by getting the URL that the latest release is redirected to
# This approach is less brittle than making a request to the GitHub API (which
# is rate limited) and parsing the response
###############################################################################
if [[ "$VERSION" == "latest" ]]; then
  info "Determining latest version..."
  
  # Function to get the latest version, with retry logic
  get_latest_version() {
    local retries=3
    local wait=2
    local attempt=1
    
    while [[ $attempt -le $retries ]]; do
      VERSION=$(curl -fsIL -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" |
        sed -E 's#.*/tag/([^[:space:]]+)#\1#')
      
      if [[ -n "$VERSION" ]]; then
        break
      fi
      
      warn "Failed to get latest version (attempt $attempt/$retries), retrying in ${wait}s..."
      sleep $wait
      wait=$((wait * 2))
      attempt=$((attempt + 1))
    done
    
    [[ -n "$VERSION" ]]
  }
  
  if ! get_latest_version; then
    error "Could not resolve latest release version. GitHub API may be rate-limited or unavailable."
  fi
  
  info "Latest version is $VERSION"
fi

###############################################################################
# Build asset URLs
###############################################################################
TAR_NAME="cli-${VERSION}-${TARGET}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
TAR_URL="${BASE_URL}/${TAR_NAME}"
SHA_URL="${TAR_URL}.sha256"

###############################################################################
# Download, verify, extract
###############################################################################
echo "⬇️  Downloading ${BIN_NAME} ${VERSION} for ${TARGET}..."
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM HUP

# Download with progress indicator for larger files
# Use -# for progress bar in non-silent mode
if ! curl -fL --connect-timeout 15 --retry 3 --retry-delay 2 \
    -# "$TAR_URL" -o "${TMP_DIR}/${TAR_NAME}"; then
error "Failed to download asset: $TAR_URL"
fi


# Extract archive
if ! tar --strip-components=1 -xzf "${TMP_DIR}/${TAR_NAME}" -C "$TMP_DIR" 2>/dev/null; then
error "Failed to extract archive. The file may be corrupted or in an unexpected format."
fi

###############################################################################
# Install
###############################################################################
BIN_SRC="${TMP_DIR}/${BIN_NAME}"
[[ -f "$BIN_SRC" ]] || error "Binary not found in archive"

if [[ -w "$INSTALL_DIR" ]]; then
  install -m 755 "$BIN_SRC" "$INSTALL_DIR/$BIN_NAME" || 
    error "Failed to install binary to $INSTALL_DIR"
else
  info "Installation directory '$INSTALL_DIR' requires elevated privileges (sudo)..."
  sudo install -m 755 "$BIN_SRC" "$INSTALL_DIR/$BIN_NAME" || 
    error "Failed to install binary (sudo) to $INSTALL_DIR"
fi

###############################################################################
# Post‑install verification
###############################################################################
if [[ -x "$INSTALL_DIR/$BIN_NAME" ]]; then
  success "Installed '${BIN_NAME}' ${VERSION} to ${INSTALL_DIR}"
  
  # Test if command is in PATH
  if ! check_command "$BIN_NAME"; then
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
      info "Note: '${INSTALL_DIR}' is not on your PATH."
      
      # Determine appropriate shell config file
      SHELL_CONFIG=""
      SHELL_NAME=$(basename "$SHELL")
      case "$SHELL_NAME" in
        bash) SHELL_CONFIG="~/.bashrc or ~/.bash_profile" ;;
        zsh)  SHELL_CONFIG="~/.zshrc" ;;
        fish) SHELL_CONFIG="~/.config/fish/config.fish" ;;
        *)    SHELL_CONFIG="your shell's configuration file" ;;
      esac
      
      # Offer to modify PATH
      echo
      read -r -p "Would you like to add $INSTALL_DIR to your PATH? [y/N] " response
      if [[ "${response,,}" =~ ^(yes|y)$ ]]; then
        case "$SHELL_NAME" in
          bash|zsh)
            echo "export PATH=\"\$PATH:${INSTALL_DIR}\"" >> "${HOME}/.${SHELL_NAME}rc"
            info "Added to ${HOME}/.${SHELL_NAME}rc - restart your shell or run 'source ${HOME}/.${SHELL_NAME}rc'"
            ;;
          fish)
            mkdir -p ~/.config/fish
            echo "set -gx PATH \$PATH ${INSTALL_DIR}" >> ~/.config/fish/config.fish
            info "Added to ~/.config/fish/config.fish - restart your shell or run 'source ~/.config/fish/config.fish'"
            ;;
          *)
            info "Please add the following line to $SHELL_CONFIG:"
            echo "    export PATH=\"\$PATH:${INSTALL_DIR}\""
            ;;
        esac
      else
        info "You can manually add it to your PATH by adding this line to $SHELL_CONFIG:"
        echo "    export PATH=\"\$PATH:${INSTALL_DIR}\""
      fi
    fi
  else
    # Verify installed version
    INSTALLED_VERSION=$("$INSTALL_DIR/$BIN_NAME" --version 2>/dev/null || echo "unknown")
    if [[ "$INSTALLED_VERSION" != "unknown" ]]; then
      success "$BIN_NAME is now available: $INSTALLED_VERSION"
    else
      warn "Installed $BIN_NAME but couldn't verify version"
    fi
  fi
  
  echo
  info "To get started, run: $BIN_NAME --help"
else
  error "Installation seems to have failed. Check permissions and try again."
fi
