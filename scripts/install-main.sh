#!/usr/bin/env bash
# Build and install the Stencila CLI from the local main branch.

set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AUTO_YES=0
SKIP_PULL=0

usage() {
  cat <<EOF
Usage: $(basename "$0") [options]

Build and install the Stencila CLI from this repository (intended for main branch).

Options:
  -y, --yes        Assume yes for all prompts
      --skip-pull  Do not pull latest changes from origin/main
  -h, --help       Show this help message
EOF
}

error() {
  echo "❌ $1" >&2
  exit 1
}

warn() {
  echo "⚠️  $1" >&2
}

info() {
  echo "ℹ️  $1"
}

success() {
  echo "✅ $1"
}

have() {
  command -v "$1" >/dev/null 2>&1
}

confirm() {
  local prompt="$1"
  local default="${2:-N}"

  if [[ "$AUTO_YES" -eq 1 ]]; then
    return 0
  fi

  local suffix
  if [[ "$default" == "Y" ]]; then
    suffix="[Y/n]"
  else
    suffix="[y/N]"
  fi

  local reply
  read -r -p "$prompt $suffix " reply
  reply="${reply:-$default}"
  [[ "${reply,,}" =~ ^(y|yes)$ ]]
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      -y|--yes)
        AUTO_YES=1
        ;;
      --skip-pull)
        SKIP_PULL=1
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        error "Unknown option: $1"
        ;;
    esac
    shift
  done
}

check_base_dependencies() {
  for cmd in git make curl; do
    have "$cmd" || error "Required command '$cmd' not found. Please install it and rerun this script."
  done
}

ensure_rust() {
  if have rustup && have cargo; then
    info "Rust toolchain is installed."
    return
  fi

  warn "Rust (via rustup) is required to build the CLI."
  if ! confirm "Install Rust now using rustup?" "Y"; then
    error "Rust is required. Install it from https://rustup.rs and rerun this script."
  fi

  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
  fi

  have rustup || error "rustup installation appears to have failed."
  have cargo || error "cargo installation appears to have failed."

  rustup toolchain install stable
  rustup default stable

  success "Rust installed successfully."
}

install_node_with_package_manager() {
  if have brew; then
    brew install node
  elif have apt-get; then
    sudo apt-get update
    sudo apt-get install -y nodejs npm
  elif have dnf; then
    sudo dnf install -y nodejs npm
  elif have pacman; then
    sudo pacman -S --noconfirm nodejs npm
  else
    return 1
  fi
}

ensure_node() {
  if have node && have npm; then
    info "Node.js is installed: $(node --version)"
    return
  fi

  warn "Node.js and npm are required (for building web assets embedded in the CLI)."
  if ! confirm "Try to install Node.js now?" "Y"; then
    error "Node.js is required. Install it from https://nodejs.org and rerun this script."
  fi

  if ! install_node_with_package_manager; then
    error "Could not auto-install Node.js (no supported package manager found). Install Node.js from https://nodejs.org and rerun this script."
  fi

  have node || error "Node.js installation appears to have failed."
  have npm || error "npm installation appears to have failed."

  success "Node.js installed successfully: $(node --version)"
}

ensure_main_branch() {
  local branch
  branch="$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || true)"

  if [[ -z "$branch" ]]; then
    warn "Could not determine current git branch. Continuing."
    return
  fi

  if [[ "$branch" != "main" ]]; then
    warn "Current branch is '$branch', not 'main'."
    if confirm "Checkout 'main' branch before building?" "Y"; then
      git -C "$REPO_ROOT" checkout main ||
        error "Unable to checkout 'main'. Commit or stash local changes and retry."
      branch="main"
    else
      warn "Proceeding with branch '$branch'."
    fi
  fi

  if [[ "$branch" == "main" && "$SKIP_PULL" -eq 0 ]]; then
    if ! git -C "$REPO_ROOT" remote get-url origin >/dev/null 2>&1; then
      warn "No 'origin' remote found; skipping pull from origin/main."
      return
    fi

    if confirm "Pull latest changes from origin/main?" "Y"; then
      git -C "$REPO_ROOT" pull --ff-only origin main ||
        error "Failed to pull latest changes. Resolve git issues and rerun."
    fi
  fi
}

build_and_install() {
  info "Building web assets..."
  make -C "$REPO_ROOT/web" clean build

  info "Installing CLI with cargo install (debug build)..."
  cargo install --debug --locked --path "$REPO_ROOT/rust/cli"

  local cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin"

  if have stencila; then
    success "Installed: $(stencila --version 2>/dev/null || echo 'stencila')"
  else
    warn "'stencila' is not currently on your PATH."
    echo "Add this directory to your PATH and restart your shell:"
    echo "  $cargo_bin"
  fi
}

main() {
  parse_args "$@"

  info "Repository root: $REPO_ROOT"
  check_base_dependencies

  # Ensure cargo is available in the current shell if Rust is already installed.
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
  fi

  ensure_rust

  # Ensure cargo is available in the current shell if Rust was just installed.
  if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
  fi

  ensure_node
  ensure_main_branch
  build_and_install

  echo
  info "Done. Run 'stencila --help' to get started."
}

main "$@"
