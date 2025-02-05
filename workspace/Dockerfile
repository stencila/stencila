FROM gitpod/openvscode-server:latest

ENV OVS_HOME="/home/.openvscode-server"
ENV OVS="${OVS_HOME}/bin/openvscode-server"

# Base image uses different so switch to root for installs
USER root

# Use bash instead of sh in the following RUNs
SHELL ["/bin/bash", "-c"]

# Customize icons
COPY icons/stencila-192.png ${OVS_HOME}/resources/server/code-192.png
COPY icons/stencila-512.png ${OVS_HOME}/resources/server/code-512.png
COPY icons/stencila-favicon.ico ${OVS_HOME}/resources/server/favicon.ico
COPY icons/stencila-icon.svg ${OVS_HOME}/out/media/code-icon.svg
COPY icons/stencila-light.svg ${OVS_HOME}/out/media/letterpress-light.svg
COPY icons/stencila-light.svg ${OVS_HOME}/out/media/letterpress-hcLight.svg
COPY icons/stencila-dark.svg ${OVS_HOME}/out/media/letterpress-dark.svg
COPY icons/stencila-dark.svg ${OVS_HOME}/out/media/letterpress-hcDark.svg

# Import the GitHub CLI GPG key and add GitHub CLI's official package repository
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    | tee /etc/apt/sources.list.d/github-cli.list

# Update & install packages
RUN apt-get update && \
    apt-get install -y \
        gh pandoc && \
    rm -rf /var/lib/apt/lists/*

# Install uv & ruff
RUN curl -LsSf https://astral.sh/uv/install.sh | sh && \
    curl -LsSf https://astral.sh/ruff/install.sh | sh

# Install latest Python using uv
RUN .local/bin/uv python install

# Preinstall extensions
RUN exts=(\
    stencila.stencila \
    streetsidesoftware.code-spell-checker \
    usernamehw.errorlens \
    ) && for ext in "${exts[@]}"; do ${OVS} --install-extension "${ext}"; done

# Add custom OpenVSCode Server settings
COPY settings.jsonc .openvscode-server/data/Machine/settings.json

# Copy the setup and entrypoint scripts
COPY setup.sh entrypoint.sh ./
RUN chmod +x setup.sh entrypoint.sh

ENTRYPOINT ["/bin/sh", "entrypoint.sh"]
