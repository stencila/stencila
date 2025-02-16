#!/usr/bin/env bash

if [[ -n "${GITHUB_REPO:-}" ]]; then
    # Create the folder that the openvscode will be opened in
    mkdir -p "$GITHUB_REPO/${REPO_SUBDIR:-}"
fi

# Start OpenVSCode Server
# This should be accessed with the query parameter ?folder=$GITHUB_REPO/$GITHUB_SUBDIR
${OVS_HOME}/bin/openvscode-server --host 0.0.0.0 --port 8080 --without-connection-token
