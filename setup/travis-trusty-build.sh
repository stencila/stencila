#!/usr/bin/env bash

# Shell script for provisioning a Travis CI Ubuntu 14.04 VM to build Stencila
# Much of this could be integrated into `../.travis.yml` but having it in a
# separate script reduces clutter there and allows for testing of this setup in Vagrant first

export DEBIAN_FRONTEND=noninteractive


# Add additional package repositories
sudo apt-get install -yq software-properties-common

sudo add-apt-repository 'deb http://cloud.r-project.org/bin/linux/ubuntu trusty/'
sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E084DAB9

sudo apt-get update


# Python

: ${PY_VERSION:=2.7}

if [[ "$PY_VERSION" == "2.7" ]]; then
	sudo apt-get install -yq --no-install-recommends --no-install-suggests \
		python2.7=$PY_VERSION.* \
		python2.7-dev=$PY_VERSION.* \
		python-pip

	pip2.7 install --user travis --upgrade pip setuptools wheel virtualenv pytest coverage awscli
else
	sudo apt-get install -yq --no-install-recommends --no-install-suggests \
		python3=$PY_VERSION.* \
		python3-dev=$PY_VERSION.* \
		python3-pip

	pip3 install --user travis --upgrade pip setuptools wheel virtualenv pytest coverage awscli
fi


# R

: ${R_VERSION:=3.3}

sudo apt-get install -yq --no-install-recommends --no-install-suggests \
	r-base-core=$R_VERSION.* \
	r-base-dev=$R_VERSION.*

sudo Rscript -e "install.packages(c('Rcpp','codetools','roxygen2','svUnit'),repo='http://cloud.r-project.org/')"
