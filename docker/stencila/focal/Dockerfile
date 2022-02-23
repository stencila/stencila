FROM ubuntu:focal

ENV STENCILA_USER_ID=1000
ENV STENCILA_GROUP_ID=1000
RUN groupadd --gid ${STENCILA_GROUP_ID} stencila \
 && useradd --uid ${STENCILA_USER_ID} --gid ${STENCILA_GROUP_ID} -m -s /bin/bash stencila

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y \
      libssl1.1 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=stencila/maker:focal /stencila/target/release/stencila /usr/local/bin/

USER stencila
WORKDIR /workspace
ENTRYPOINT ["stencila"]
