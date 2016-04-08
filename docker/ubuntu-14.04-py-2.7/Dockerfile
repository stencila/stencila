FROM ubuntu:14.04

# Turn off interactive installs
# See https://github.com/phusion/baseimage-docker/issues/58
ENV DEBIAN_FRONTEND noninteractive

# Install Python and other requirements
# System packages for scipy etc are installed here so that
# dependencies are installed but are then upgraded using pip
# Curl is required
RUN apt-get update \
  && apt-get install -y \
       python2.7 python2.7-dev python-pip \
       python-scipy python-matplotlib \
       curl \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install latest pip
RUN pip install --upgrade pip

# Install popular packages
RUN pip install --upgrade \
	numpy scipy pandas matplotlib ggplot

# Install Stencila R package
COPY stencila-latest-cp27-none-linux_x86_64.whl /tmp/stencila-0.00-cp27-none-linux_x86_64.whl
RUN pip install /tmp/stencila-0.00-cp27-none-linux_x86_64.whl

# Install Stencila session script
COPY stencila-session.py /usr/local/bin/stencila-session

# Add a user (creating home and specifying shell) and
# setup working environment
RUN useradd --create-home --shell /bin/bash stencila \
  && mkdir -p /home/stencila/store \
  && chown stencila /home/stencila/store \
  && chmod 755 /usr/local/bin/stencila-session
USER stencila
WORKDIR /home/stencila
ENV STENCILA_STORES /home/stencila/store
