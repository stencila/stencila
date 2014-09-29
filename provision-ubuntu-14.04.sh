#!/usr/bin/env bash

# Shell script for provisioning Ubuntu 14.04 with the tools
# required to build Stencila library modules

# Add necessary repositories
sudo add-apt-repository "deb http://cran.us.r-project.org/bin/linux/ubuntu trusty/"

# Install system packages used for builds and tests
sudo apt-get install \
	build-essential cmake git \
	g++ \
	libssl-dev \
	python python-dev python-pip \
	r-base r-base-dev \
  --assume-yes --fix-broken

# Install Python packages used for packaging and testing
sudo pip install wheel virtualenv

# Install R packages used for packaging and testing
sudo Rscript -e "install.packages(c('roxygen2','svUnit','XML'),lib='/usr/lib/R/library',repos='http://cran.us.r-project.org')"
