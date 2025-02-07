# Docker image for local testing of extension when running in OpenVSCode Server
# See the `make docker` recipe for more details and usage

FROM gitpod/openvscode-server:latest

ENV OVS_HOME="/home/.openvscode-server"
ENV OVS="${OVS_HOME}/bin/openvscode-server"

COPY stencila-*.vsix .
RUN ${OVS} --install-extension stencila-*.vsix

ENTRYPOINT [ "/bin/sh", "-c", "exec ${OVS} --host 0.0.0.0 --without-connection-token" ]
