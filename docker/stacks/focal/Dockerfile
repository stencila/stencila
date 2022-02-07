# Builds a CNB `stack`: base images for build-time and run-time
# See https://buildpacks.io/docs/operator-guide/create-a-stack/

# Base with required CNB environment variables and the `stencila`
# binary (and its dependencies)

FROM ubuntu:focal AS base

ENV CNB_USER_ID=1000
ENV CNB_GROUP_ID=1000
ENV CNB_STACK_ID="stencila.stacks.focal"
LABEL io.buildpacks.stack.id="stencila.stacks.focal"

RUN groupadd cnb --gid ${CNB_GROUP_ID} \
 && useradd --uid ${CNB_USER_ID} --gid ${CNB_GROUP_ID} -m -s /bin/bash cnb

ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y \
      libssl1.1 \
 && rm -rf /var/lib/apt/lists/*

COPY --from=stencila/builder:focal /stencila/target/release/stencila /usr/local/bin/


# Stack `run` stage

FROM base AS run

USER ${CNB_USER_ID}:${CNB_GROUP_ID}


# Stack `build` stage

FROM base AS build

USER ${CNB_USER_ID}:${CNB_GROUP_ID}
