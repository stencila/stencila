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

# Import the CRAN GPG key and add CRAN's official package repository
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 51716619E084DAB9 && \
    echo "deb https://cloud.r-project.org/bin/linux/ubuntu jammy-cran40/" \
    | tee /etc/apt/sources.list.d/cran.list

# Update & install packages
RUN apt-get update && \
    apt-get install -y \
        gh pandoc r-base && \
    rm -rf /var/lib/apt/lists/*

# Install uv
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/home/workspace/.local/bin:$PATH"

# Install Python and tools using uv
# This places ruff and pyright in /home/workspace/.local/bin with is on $PATH
RUN uv python install && \
    uv tool install ruff && \
    uv tool install pyright[nodejs]

# Install R package management tools
RUN Rscript -e "install.packages(c('pak', 'renv'))"
ENV RENV_CONFIG_PAK_ENABLED=TRUE

# Preinstall VSCode extensions
RUN exts=(\
    stencila.stencila \
    streetsidesoftware.code-spell-checker \
    usernamehw.errorlens \
    ) && for ext in "${exts[@]}"; do ${OVS} --install-extension "${ext}"; done

# Add custom OpenVSCode Server settings
COPY settings.jsonc .openvscode-server/data/Machine/settings.json

# Copy over Stencila workspace files
COPY defaults/* .stencila/workspace/

# Install default packages for Python and R so that they are cached in the image
RUN cd .stencila/workspace && \
    uv sync && \
    Rscript -e "renv::install()"

# Copy over and specify the entrypoint script
COPY entrypoint.sh .
ENTRYPOINT ["/bin/bash", "entrypoint.sh"]
