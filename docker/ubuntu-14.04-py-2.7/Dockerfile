FROM ubuntu:14.04

# Turn off interactive installs
# See https://github.com/phusion/baseimage-docker/issues/58
ENV DEBIAN_FRONTEND noninteractive

# Install Python and other requirements
# The lines following the `python2.7..` line are for installing requirements
# for scipy, matplotlib
RUN apt-get update \
  && apt-get install -y \
    python2.7 python2.7-dev python-pip \
    libblas-dev liblapack-dev gfortran \
    libfreetype6-dev libpng-dev pkg-config libcairo2 libcairo2-dev libffi6 libffi-dev \
    curl \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install latest pip
RUN pip install --upgrade pip

# Install popular packages
RUN pip install --upgrade \
	numpy scipy pandas cairocffi matplotlib ggplot==0.6.8

# Install Stencila R package
COPY stencila.whl /tmp/stencila-0.00-cp27-none-linux_x86_64.whl
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
