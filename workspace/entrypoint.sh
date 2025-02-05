#!/bin/bash

# Create a folder for the GitHub repo and copy the setup script to the expected
# place so that it is run by the Stencila VSCode extension when it starts up
if [ ! -z "$GITHUB_REPO" ]; then
    mkdir -p "$GITHUB_REPO/.stencila/workspace"
    cp setup.sh "$GITHUB_REPO/.stencila/workspace"
    echo "workspace/setup.sh" > "$GITHUB_REPO/.stencila/.gitignore"
fi

# Start OpenVSCode Server
${OVS_HOME}/bin/openvscode-server --host 0.0.0.0 --port 8080 --without-connection-token
