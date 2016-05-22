FROM ubuntu:14.04

# Turn off interactive installs
# See https://github.com/phusion/baseimage-docker/issues/58
ENV DEBIAN_FRONTEND noninteractive

# Install R
RUN apt-get update \
  && apt-get install -y software-properties-common \
  && apt-add-repository 'deb http://cran.us.r-project.org/bin/linux/ubuntu trusty/' \
  && apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E084DAB9 \
  && apt-get update \
  && apt-get install -y \
  	r-base-core=3.2.5-1trusty0 \
  	r-base-dev=3.2.5-1trusty0 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

# Install Stencila R package dependencies and some popular packages
RUN Rscript -e "install.packages(c('Rcpp','codetools','ggplot2','stringr','plyr','dplyr','reshape2'),repo='http://cran.us.r-project.org/')"

# Install Stencila R package
COPY stencila.tar.gz /tmp/stencila.tar.gz
RUN R CMD INSTALL /tmp/stencila.tar.gz \
  && Rscript -e 'require(stencila); stencila:::install()'

# Install Stencila session script
COPY stencila-session.r /usr/local/bin/stencila-session

# Add a user (creating home and specifying shell) and
# setup working environment
RUN useradd --create-home --shell /bin/bash stencila \
  && mkdir -p /home/stencila/store \
  && chown stencila /home/stencila/store \
  && chmod 755 /usr/local/bin/stencila-session
USER stencila
WORKDIR /home/stencila
ENV STENCILA_STORES /home/stencila/store
