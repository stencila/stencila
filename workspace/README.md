# Stencila Workspace

**A Docker image for Stencila workspace sessions**

## Use Cases

This workspace is designed for two primary purposes:

1. **Cloud Development Environments (CDE)**: Provides users with pre-configured, browser-accessible environments for working on Stencila projects without local setup requirements.

2. **Automation and Sync Workflows**: Powers Stencila's sync feature which enables bidirectional synchronization between Git repositories and cloud office tools (Google Docs, Microsoft 365, Notion).

## Pre-installed Components

### Development Tools

- `build-essential` (C/C++ compilers and build tools for native extensions)
- [GitHub CLI](https://cli.github.com/)
- [`mise`](https://mise.jdx.dev/) (tool version manager)
- [Pandoc](https://pandoc.org/) (document converter)

### Python Ecosystem

- [`uv`](https://docs.astral.sh/uv/) (fast Python package manager)
- [`ruff`](https://docs.astral.sh/ruff/) (linter and formatter)
- `pyright` (type checker)
- Common packages: `pandas`, `polars`, `matplotlib`, `seaborn`, `plotly`, `altair`, `plotnine`

### R Ecosystem

- [`lintr`](https://github.com/r-lib/lintr/) (linter)
- [`styler`](https://github.com/r-lib/styler) (code formatter)
- `pak` and `renv` (package management)
- Common packages: `tidyverse`, `data.table`, `Cairo`

### VSCode and Extensions

- [`openvscode-server`](https://github.com/gitpod-io/openvscode-server)
- [Stencila VSCode extension](https://marketplace.visualstudio.com/items?itemName=stencila.stencila)
- [Code Spell Checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker)

## How It Works

The workspace supports two distinct operational modes:

- If `STENCILA_SCRIPT` environment variable is set → CI Mode
- If `STENCILA_SCRIPT` is not set → CDE Mode (default)

### 1. CDE Mode (Default)

Interactive development environment with browser-accessible VSCode.

**Architecture:**
```
User Browser
    ↓
entrypoint.sh → OpenVSCode Server (port 8080)
    ↓
Stencila VSCode Extension
    ↓
setup.sh (if present in workspace)
    ↓
User Workspace (with dependencies installed)
```

**Workflow:**
1. The container starts via `entrypoint.sh`, launching OpenVSCode Server on port 8080
2. The browser connects to the server (typically at `http://localhost:8080/?folder=/path/to/workspace`)
3. If a `setup.sh` file exists in the workspace root, the Stencila VSCode extension automatically detects and runs it (see [`vscode/src/workspace.ts`](../vscode/src/workspace.ts))
4. Users can see setup progress directly in the VSCode window

### 2. CI Mode

Headless automation for running scripts (sync, testing, deployment, etc.).

**Architecture:**
```
entrypoint.sh
    ↓
setup.sh (initializes repository)
    ↓
specified script (e.g., sync-to-remote.sh, test-runner.sh)
    ↓
Stencila CLI / External Services
```

**Workflow:**
1. The container starts via `entrypoint.sh` with `STENCILA_SCRIPT` environment variable set
2. `setup.sh` is executed to clone and configure the Git repository
3. The specified script from `/home/workspace/stencila/defaults/` executes
4. The container exits with the script's exit code

### Container User

The container runs as the `stencila` user (UID 1000, GID 1000) for improved security. System packages and OpenVSCode Server are installed as root during the build, but all user-level tools (mise, uv, Python/R packages, VSCode extensions) and runtime processes execute as the non-root `stencila` user. This user has passwordless sudo access for administrative tasks when needed.

The workspace directory is located at `/home/workspace` with the `stencila` user as owner.

### Environment Variables

#### Common Variables (Both Modes)

- `GITHUB_REPO`: GitHub repository to clone (format: `owner/repo`)
- `REPO_SUBDIR`: Subdirectory within the repository to work in (optional)
- `REPO_REF`: Git reference to checkout - branch, tag, or commit (optional)
- `GITHUB_TOKEN`: GitHub authentication token for private repositories (optional)

#### CI Mode Variables

- `STENCILA_SCRIPT`: Name of the script to run from `/home/workspace/stencila/defaults/` (e.g., `sync-to-remote.sh`, `sync-from-remote.sh`, `test-runner.sh`)

**Sync Script Variables** (when using sync scripts):
- `STENCILA_SYNC_FILE_PATH`: Path to the local file to sync (relative to repository directory)
- `STENCILA_SYNC_REMOTE_URL`: URL of the remote cloud document (e.g., Google Docs URL, OneDrive URL)

Note: Authentication for cloud services is handled by the Stencila CLI using credentials configured in your Stencila account.

### Default Dependencies and Scripts

The `defaults/` directory contains fallback package specifications and sync scripts:

**Package Specifications:**
- **`pyproject.toml`**: Python dependencies (requires Python ≥3.12)
- **`DESCRIPTION`**: R package metadata

**Initialization and Sync Scripts:**
- **`setup.sh`**: Workspace initialization script (clones repo, installs dependencies)
- **`sync-to-remote.sh`**: Pushes local file to remote cloud document using `stencila push`
- **`sync-from-remote.sh`**: Pulls remote cloud document to local file using `stencila pull`, then commits and pushes to Git

The package specifications serve two purposes:

1. **Build-time caching**: Packages are pre-installed during Docker image build for faster workspace startup
2. **Runtime fallback**: If a workspace doesn't specify its own dependencies, the defaults are used

The scripts are executed in CI mode when specified via the `STENCILA_SCRIPT` environment variable.

## Configuration Files

### `settings.jsonc`

Machine-level VSCode settings:

- Sets "Stencila" window title and "Stencila Light" theme
- Configures Python interpreter to use workspace virtual environment
- Enables word wrap for `.smd` files (Stencila Markdown)
- Disables minimap and other UI customizations

Copied to `.openvscode-server/data/Machine/settings.json` in the container.

## Security Considerations

**⚠️ Important Security Notice**

This workspace runs OpenVSCode Server with `--without-connection-token`, meaning **no authentication is required by default**.

This configuration is intended for:

- Running behind an authentication proxy or gateway
- Use in trusted networks only
- Local development environments

**If exposing this workspace publicly, you must add your own authentication layer.** Do not expose an unauthenticated instance to the internet.

## Development Workflow

### Building and Testing

The `Makefile` provides convenient targets:

```bash
# Lint shell scripts
make lint

# Build Docker image
make build

# Build without cache (clean build)
make build-no-cache

# Build and run server on port 8080
make run

# Enter container shell for debugging
make debug

# Tag with timestamp and push to Docker Hub
make publish
```

### Build Process

The Dockerfile follows a carefully ordered build process optimized for layer caching:

1. **Base setup**: Ubuntu 24.04 with core system packages (curl, wget, git, sudo, gnupg)
2. **OpenVSCode Server**: Download and install release from gitpod-io
3. **Branding**: Customize OpenVSCode icons with Stencila branding
4. **User configuration**: Create `stencila` user (UID 1000) with appropriate permissions
5. **Repository setup**: Add GitHub CLI and CRAN package repositories with GPG keys
6. **Development tools**: Install build-essential, GitHub CLI, Pandoc, R base
7. **User switch**: Switch to non-root `stencila` user for security
8. **Tool installation**: Install mise, uv, Python tools (ruff, pyright), R tools (pak, renv) as non-root user
9. **VSCode extensions**: Pre-install Stencila and utility extensions
10. **Stencila CLI**: Symlink CLI from extension to PATH
11. **Default dependencies**: Pre-install Python/R packages from `defaults/` directory
12. **Configuration**: Add Stencila config and VSCode settings (placed late for optimal cache reuse)
13. **Entrypoint**: Copy startup script

The build is optimized with frequently-changing files (config, settings, entrypoint) placed near the end to maximize Docker layer cache reuse. Images are built for `linux/amd64` and published to Docker Hub as `stencila/workspace`.

## Usage Examples

### CDE Mode

#### Running Locally

```bash
docker run -p 8080:8080 stencila/workspace
```

Then open http://localhost:8080 in your browser.

#### With GitHub Repository

```bash
docker run -p 8080:8080 \
  -e GITHUB_REPO="owner/repo" \
  -e REPO_SUBDIR="subdir" \
  -e GITHUB_TOKEN="ghp_xxxx" \
  stencila/workspace
```

Then open http://localhost:8080/?folder=/workspace/owner/repo/subdir in your browser.

### CI Mode

#### Example: Sync to Remote (Push)

Push a local file to a cloud document (e.g., Google Docs, OneDrive):

```bash
docker run \
  -e STENCILA_SCRIPT="sync-to-remote.sh" \
  -e GITHUB_REPO="owner/repo" \
  -e GITHUB_TOKEN="ghp_xxxx" \
  -e STENCILA_SYNC_FILE_PATH="docs/report.md" \
  -e STENCILA_SYNC_REMOTE_URL="https://docs.google.com/document/d/abc123..." \
  stencila/workspace
```

#### Example: Sync from Remote (Pull)

Pull changes from a cloud document and push them to the Git repository:

```bash
docker run \
  -e STENCILA_SCRIPT="sync-from-remote.sh" \
  -e GITHUB_REPO="owner/repo" \
  -e GITHUB_TOKEN="ghp_xxxx" \
  -e STENCILA_SYNC_FILE_PATH="docs/report.md" \
  -e STENCILA_SYNC_REMOTE_URL="https://onedrive.live.com/edit.aspx?..." \
  stencila/workspace
```
