#!/usr/bin/env bash

# Shell script for provisioning a Travis CI Ubuntu 14.04 VM to build Stencila
# Much of this could be integrated into `../.travis.yml` but having it in a
# separate script reduces clutter there and allows for testing of this setup in Vagrant first

export DEBIAN_FRONTEND=noninteractive


# Add additional package repositories
sudo apt-get install -yq software-properties-common

sudo add-apt-repository 'deb http://cran.us.r-project.org/bin/linux/ubuntu trusty/' \
  && apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E084DAB9

sudo apt-get update


# Python


# R

: ${R_VERSION:=3.3}

sudo apt-get install -yq --no-install-recommends --no-install-suggests \
	r-base-core=$R_VERSION* \
	r-base-dev=$R_VERSION*

sudo Rscript -e "install.packages(c('Rcpp','roxygen2','svUnit'),repo='http://cran.us.r-project.org/')"
