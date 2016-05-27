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

# Node
# See https://github.com/travis-ci/travis-ci/issues/2311#issuecomment-171180704
# and https://github.com/mapbox/node-pre-gyp#travis-os-x-gochas

: ${NODE_VERSION:=4.4}

rm -rf ~/.nvm/ && git clone --depth 1 https://github.com/creationix/nvm.git ~/.nvm
source ~/.nvm/nvm.sh
nvm install $NODE_VERSION
nvm use $NODE_VERSION
nvm alias default $NODE_VERSION

# Python

: ${PY_VERSION:=2.7}

if [[ "$PY_VERSION" == "2.7" ]]; then
	PY_PACKAGE=python2.7
	PY_PIP_PACKAGE=python-pip
	PY_PIP=pip2.7
else
	PY_PACKAGE=python3
	PY_PIP_PACKAGE=python3-pip
	PY_PIP=pip3
fi

sudo apt-get install -yq --no-install-recommends --no-install-suggests \
	$PY_PACKAGE=$PY_VERSION.* \
	$PY_PACKAGE-dev=$PY_VERSION.* \
	$PY_PIP_PACKAGE
$PY_PIP install --user travis --upgrade pip setuptools wheel virtualenv tox awscli

# R

: ${R_VERSION:=3.3}

sudo apt-get install -yq --no-install-recommends --no-install-suggests \
	r-base-core=$R_VERSION.* \
	r-base-dev=$R_VERSION.*

sudo Rscript -e "install.packages(c('Rcpp','codetools','roxygen2','svUnit'),repo='http://cloud.r-project.org/')"
