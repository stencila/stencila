#!/usr/bin/env bash

set -euo pipefail

# Function to call when the container exits
cleanup() {
    local exit_code=$?

    # Determine status based on exit code and status file
    local status="succeeded"
    if [[ $exit_code -ne 0 ]]; then
        status="failed"
    elif [[ -f /tmp/stencila-status ]]; then
        status=$(cat /tmp/stencila-status)
        rm -f /tmp/stencila-status
    fi

    # Only call the API if STENCILA_SESSION_ID and STENCILA_API_TOKEN are set
    if [[ -n "${STENCILA_SESSION_ID:-}" && -n "${STENCILA_API_TOKEN:-}" ]]; then
        local api_url="${STENCILA_API_URL:-https://api.stencila.cloud}"
        local url="${api_url}/v1/sessions/${STENCILA_SESSION_ID}/finished?status=${status}"
        local max_attempts=3
        local attempt=1
        local success=false

        echo "📡 Notifying Stencila Cloud that session $status"

        while [[ $attempt -le $max_attempts ]]; do
            if curl -f -s -o /dev/null -X POST "$url" \
                -H "Authorization: Bearer ${STENCILA_API_TOKEN}" \
                --connect-timeout 10 \
                --max-time 30; then
                success=true
                break
            fi

            if [[ $attempt -lt $max_attempts ]]; then
                local delay=$((attempt * 2))
                echo "🔄 Retry $attempt/$max_attempts failed, waiting ${delay}s..."
                sleep $delay
            fi
            ((attempt++))
        done

        if [[ "$success" != "true" ]]; then
            echo "⚠️ Warning: Failed to notify of session completion after $max_attempts attempts"
        fi
    fi
}

# Register cleanup function to run on script exit
trap cleanup EXIT

# Check if running in CI mode (script execution) or CDE mode (VSCode server)
if [[ -n "${STENCILA_SCRIPT:-}" ]]; then
    # CI mode: run the specified script
    echo "🚀 Running in CI mode with script: ${STENCILA_SCRIPT}"

    # Create the repository directory structure if GITHUB_REPO is set
    if [[ -n "${GITHUB_REPO:-}" ]]; then
        mkdir -p "$GITHUB_REPO"
        cd "$GITHUB_REPO"
    fi

    # Run setup.sh to initialize the repository
    SETUP_SCRIPT="/home/workspace/stencila/defaults/setup.sh"
    if [[ -f "${SETUP_SCRIPT}" ]]; then
        echo "🔧 Initializing workspace..."
        bash "${SETUP_SCRIPT}"
    else
        echo "⚠️ Warning: setup.sh not found at ${SETUP_SCRIPT}"
    fi

    # Run the specified script (unless it's "none")
    if [[ "${STENCILA_SCRIPT}" != "none" ]]; then
        SCRIPT_PATH="/home/workspace/stencila/defaults/${STENCILA_SCRIPT}"
        if [[ -f "${SCRIPT_PATH}" ]]; then
            echo "▶️ Executing ${STENCILA_SCRIPT}..."
            bash "${SCRIPT_PATH}"
            exit $?
        else
            echo "❌ Error: Script not found at ${SCRIPT_PATH}"
            exit 1
        fi
    else
        echo "✅ Setup complete. No script to execute (STENCILA_SCRIPT=none)."
        exit 0
    fi
else
    # CDE mode: start VSCode server
    echo "💻 Running in CDE mode (VSCode server)"

    if [[ -n "${GITHUB_REPO:-}" ]]; then
        # Create the folder that the openvscode will be opened in
        mkdir -p "$GITHUB_REPO/${REPO_SUBDIR:-}"
    fi

    # Start OpenVSCode Server
    # This should be accessed with the query parameter ?folder=$GITHUB_REPO/$GITHUB_SUBDIR
    ${OVS_HOME}/bin/openvscode-server --host 0.0.0.0 --port 8080 --without-connection-token
fi
