# A Docker image for using Stencila with Node.js
#
# Extends base Stencila image with Node.js and `jesta`.
# Intended primarily for executing documents containing Javascript code.

FROM stencila/stencila

USER root

RUN curl -sL https://deb.nodesource.com/setup_14.x | bash \
 && apt-get install -y nodejs \
 && npm config --global set user root \
 && rm -rf /var/lib/apt/lists/*

USER guest

RUN stencila plugins install jesta
