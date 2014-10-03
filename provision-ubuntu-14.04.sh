#!/usr/bin/env bash

# Shell script for provisioning Ubuntu 14.04 with the tools
# required to build Stencila library modules

###############################################################################
# Add package repositories

# Python
# To build the Stencila Python module for a particular version of Python it is necessary to 
# have the `python<version>-dev` package installed. The "deadsnakes" repository has
# packages for those versions of Python, both older and newer, which are not available
# in the official repositories.
sudo apt-add-repository 'deb http://ppa.launchpad.net/fkrull/deadsnakes/ubuntu trusty main'

# R
# Add CRAN repository for most recent version
sudo add-apt-repository "deb http://cran.us.r-project.org/bin/linux/ubuntu trusty/"

# Update
sudo apt-get update

###############################################################################
# C++ module

sudo apt-get install build-essential cmake git g++ libssl-dev --assume-yes --fix-broken

###############################################################################
# Python module	

sudo apt-get install python python-dev python-pip --assume-yes --fix-broken

# Uncomment out the following line to install multiple versions of Python
# sudo apt-get install python2.7-dev python3.0-dev python3.1-dev python3.2-dev

# Install `wheel` for packaging and `virtualenv` for testing
sudo pip install wheel virtualenv

###############################################################################
# R module

sudo apt-get install r-base r-base-dev --assume-yes --fix-broken

# Install `roxygen2` for packaging and `svUnit`, `XML` for testing
sudo Rscript -e "install.packages(c('roxygen2','svUnit','XML'),lib='/usr/lib/R/library',repos='http://cran.us.r-project.org')"
