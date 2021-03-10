# A base Docker image for Stencila
#
# Designed as a base from which other Stencila images can be built on top of.
# Uses the Stencila command line tool as it's entrypoint.
#
# Get help:
#
#    docker run -it --rm stencila/stencila help
#
# Execute a stencil in the current working directory:
#
#    docker run -it --rm -v$PWD:/work stencila/stencila execute
#
# Serve over HTTP and Websockets on port 9000:
#
#    docker run -it --rm -p9000:9000 stencila/stencila serve ws://0.0.0.0:9000

FROM ubuntu:20.04

RUN apt-get update \
 && apt-get install -y \
       curl \
       libssl1.1 \
 && rm -rf /var/lib/apt/lists/*

RUN curl -sL https://github.com/stencila/stencila/releases/download/v0.42.0/stencila-v0.42.0-x86_64-unknown-linux-gnu.tar.gz | tar xvz \
 && mv stencila* /usr/local/bin 

RUN useradd -m guest
USER guest
WORKDIR /work

ENTRYPOINT ["stencila"]
