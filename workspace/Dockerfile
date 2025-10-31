FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG OVS_RELEASE=openvscode-server-v1.97.2
ARG USERNAME=stencila
ARG USER_UID=1000
ARG USER_GID=${USER_UID}

ENV DEBIAN_FRONTEND=${DEBIAN_FRONTEND}
ENV OVS_HOME="/home/.openvscode-server"
ENV OVS="${OVS_HOME}/bin/openvscode-server"

# Metadata labels
LABEL org.opencontainers.image.title="Stencila Workspace" \
      org.opencontainers.image.description="Stencila's cloud development environment" \
      org.opencontainers.image.vendor="Stencila" \
      org.opencontainers.image.url="https://github.com/stencila/stencila/tree/main/workspace#readme" \
      org.opencontainers.image.source="https://github.com/stencila/stencila/tree/main/workspace"

# Install core packages required for OpenVSCode Server and subsequent tooling
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        wget \
        git \
        sudo \
        gnupg \
        dirmngr \
        libatomic1 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /home

# Download and install OpenVSCode Server release
RUN set -eux; \
    if [ -z "${OVS_RELEASE}" ]; then \
        echo "The OVS_RELEASE build arg must be set." >&2; \
        exit 1; \
    fi; \
    arch="$(uname -m)"; \
    case "${arch}" in \
        x86_64) arch="x64" ;; \
        aarch64) arch="arm64" ;; \
        armv7l) arch="armhf" ;; \
        *) echo "Unsupported architecture: ${arch}" >&2; exit 1 ;; \
    esac; \
    wget "https://github.com/gitpod-io/openvscode-server/releases/download/${OVS_RELEASE}/${OVS_RELEASE}-linux-${arch}.tar.gz"; \
    tar -xzf "${OVS_RELEASE}-linux-${arch}.tar.gz"; \
    mv "${OVS_RELEASE}-linux-${arch}" "${OVS_HOME}"; \
    cp "${OVS_HOME}/bin/remote-cli/openvscode-server" "${OVS_HOME}/bin/remote-cli/code"; \
    rm "${OVS_RELEASE}-linux-${arch}.tar.gz"

# Customize OpenVSCode Server icons with Stencila branding
COPY icons/ /tmp/stencila-icons/
RUN mv /tmp/stencila-icons/stencila-192.png ${OVS_HOME}/resources/server/code-192.png && \
    mv /tmp/stencila-icons/stencila-512.png ${OVS_HOME}/resources/server/code-512.png && \
    mv /tmp/stencila-icons/stencila-favicon.ico ${OVS_HOME}/resources/server/favicon.ico && \
    mv /tmp/stencila-icons/stencila-icon.svg ${OVS_HOME}/out/media/code-icon.svg && \
    mv /tmp/stencila-icons/stencila-light.svg ${OVS_HOME}/out/media/letterpress-light.svg && \
    cp ${OVS_HOME}/out/media/letterpress-light.svg ${OVS_HOME}/out/media/letterpress-hcLight.svg && \
    mv /tmp/stencila-icons/stencila-dark.svg ${OVS_HOME}/out/media/letterpress-dark.svg && \
    cp ${OVS_HOME}/out/media/letterpress-dark.svg ${OVS_HOME}/out/media/letterpress-hcDark.svg && \
    rm -rf /tmp/stencila-icons

# Configure the stencila user
RUN set -eux; \
    if getent group "${USER_GID}" >/dev/null; then \
        existing_group="$(getent group "${USER_GID}" | cut -d: -f1)"; \
        if [ "${existing_group}" != "${USERNAME}" ]; then \
            groupmod -n "${USERNAME}" "${existing_group}"; \
        fi; \
    else \
        groupadd --gid "${USER_GID}" "${USERNAME}"; \
    fi; \
    if getent passwd "${USER_UID}" >/dev/null; then \
        existing_user="$(getent passwd "${USER_UID}" | cut -d: -f1)"; \
        if [ "${existing_user}" != "${USERNAME}" ]; then \
            usermod -l "${USERNAME}" "${existing_user}"; \
        fi; \
        usermod -g "${USER_GID}" "${USERNAME}"; \
        usermod -d "/home/${USERNAME}" -m "${USERNAME}"; \
        usermod -s /bin/bash "${USERNAME}"; \
    elif id -u "${USERNAME}" >/dev/null 2>&1; then \
        usermod -u "${USER_UID}" "${USERNAME}"; \
        usermod -g "${USER_GID}" "${USERNAME}"; \
        usermod -d "/home/${USERNAME}" -m "${USERNAME}"; \
        usermod -s /bin/bash "${USERNAME}"; \
    else \
        useradd --uid "${USER_UID}" --gid "${USER_GID}" -m -s /bin/bash "${USERNAME}"; \
    fi; \
    echo "${USERNAME} ALL=(root) NOPASSWD:ALL" > /etc/sudoers.d/${USERNAME}; \
    chmod 0440 /etc/sudoers.d/${USERNAME} && \
    chmod g+rw /home && \
    mkdir -p /home/workspace && \
    chown -R ${USERNAME}:${USERNAME} /home/workspace && \
    chown -R ${USERNAME}:${USERNAME} ${OVS_HOME} && \
    chown -R ${USERNAME}:${USERNAME} "/home/${USERNAME}"

ENV LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    HOME=/home/workspace \
    EDITOR=code \
    VISUAL=code \
    GIT_EDITOR="code --wait" \
    OPENVSCODE_SERVER_ROOT=${OVS_HOME}

WORKDIR /home/workspace

# Use bash for subsequent RUN commands
SHELL ["/bin/bash", "-c"]

# Import GPG keys and add package repositories for GitHub CLI and CRAN
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    | tee /etc/apt/sources.list.d/github-cli.list && \
    export GNUPGHOME="$(mktemp -d)" && \
    chmod 700 "${GNUPGHOME}" && \
    gpg --batch --homedir "${GNUPGHOME}" --no-default-keyring --keyring /usr/share/keyrings/cran-archive-keyring.gpg --keyserver keyserver.ubuntu.com --recv-keys 51716619E084DAB9 && \
    rm -rf "${GNUPGHOME}" && \
    echo "deb [signed-by=/usr/share/keyrings/cran-archive-keyring.gpg] https://cloud.r-project.org/bin/linux/ubuntu noble-cran40/" \
    | tee /etc/apt/sources.list.d/cran.list

# Update & install packages, then prepare R library directory for non-root user
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        gh \
        pandoc \
        r-base && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* && \
    mkdir -p /usr/local/lib/R/site-library && \
    chmod -R 0777 /usr/local/lib/R/site-library

# Switch to non-root user for remaining installations
# This improves security and ensures user-level tools are installed in the correct location
# All subsequent RUN commands execute as ${USERNAME} (stencila)
USER ${USERNAME}

# Install mise (tool version manager) and uv (Python package manager)
RUN curl -LsSf https://mise.run | sh && \
    curl -LsSf https://astral.sh/uv/install.sh | sh

# Update PATH for above installs
ENV PATH="/home/workspace/.local/bin:${PATH}"

# Install Python and tools using uv
# This places ruff and pyright in /home/workspace/.local/bin which is on $PATH
RUN uv python install && \
    uv tool install ruff && \
    uv tool install pyright[nodejs] && \
    rm -rf /tmp/* ~/.cache/*

# Install R package management tools
RUN Rscript -e "install.packages(c('pak', 'renv'))"
ENV RENV_CONFIG_PAK_ENABLED=TRUE

# Preinstall VSCode extensions
RUN exts=(\
    stencila.stencila \
    streetsidesoftware.code-spell-checker \
    ) && for ext in "${exts[@]}"; do \
        ${OVS} --install-extension "${ext}" || \
        (echo "Failed to install extension ${ext}" && exit 1); \
    done

# Make the Stencila CLI installed with the extension more readily available to the user
RUN STENCILA_DIR=$(find /home/workspace/.openvscode-server/extensions -maxdepth 1 -type d -name 'stencila.stencila-*' | head -n 1) && \
    ln -s "$STENCILA_DIR/cli/stencila" /home/workspace/.local/bin/stencila

# Copy over dependency specification files first to maximize cache reuse
# Only copy files needed for package installation - changes to other files (like *.sh scripts)
# won't invalidate this layer and trigger expensive package reinstallation
RUN mkdir -p /home/workspace/stencila/defaults
COPY --chown=${USERNAME}:${USERNAME} defaults/pyproject.toml defaults/DESCRIPTION /home/workspace/stencila/defaults/

# Install default packages for Python and R so that they are cached in the image
# Use increased HTTP timeout (300s) to handle slow connections during Docker builds
RUN cd /home/workspace/stencila/defaults && \
    UV_HTTP_TIMEOUT=300 uv sync && \
    Rscript -e "renv::install()"

# Copy remaining workspace default files (e.g. scripts) after package installation
# This prevents changes to these files from invalidating the package installation layer
COPY --chown=${USERNAME}:${USERNAME} defaults/*.sh /home/workspace/stencila/defaults/

# Add Stencila user level config
# COPY creates parent directories automatically, no mkdir needed
COPY --chown=${USERNAME}:${USERNAME} config.yaml /home/workspace/.config/stencila/

# Add OpenVSCode Server settings
# Placed near the end for optimal cache invalidation - these settings change frequently
# during development but don't affect any subsequent layers, making iteration fast
COPY --chown=${USERNAME}:${USERNAME} settings.jsonc .openvscode-server/data/Machine/settings.json

# Copy entrypoint script last
# This is the most frequently modified file during development, so keeping it last
# ensures changes don't invalidate any cached layers above
COPY entrypoint.sh .

EXPOSE 8080

ENTRYPOINT ["/bin/bash", "entrypoint.sh"]
