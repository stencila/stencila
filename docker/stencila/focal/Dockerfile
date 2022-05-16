FROM ubuntu:focal

ENV STENCILA_USER_ID=1000
ENV STENCILA_GROUP_ID=1000
RUN groupadd --gid ${STENCILA_GROUP_ID} stencila \
 && useradd --uid ${STENCILA_USER_ID} --gid ${STENCILA_GROUP_ID} -m -s /bin/bash stencila

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y \
      ca-certificates \
      curl \
      openssl \
 && rm -rf /var/lib/apt/lists/*

RUN curl -sL https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash -s v1.9.0 /usr/bin

USER stencila
WORKDIR /workspace
ENTRYPOINT ["stencila"]
