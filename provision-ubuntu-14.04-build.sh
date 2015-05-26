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

sudo apt-get install build-essential cmake git g++ libssl-dev dos2unix --force-yes --assume-yes --fix-broken

###############################################################################
# Javascript module	
# `uglifyjs` for minification; `phantomjs` is required for testing

sudo apt-get install nodejs --force-yes --assume-yes --fix-broken
sudo npm install -g uglify-js

# Currently an official binary of PhantomJs 2.0 is not available
# So use a contributed binary instead (compile times are very long!)
# See https://github.com/ariya/phantomjs/issues/12948
cd /tmp
wget  --no-check-certificate -O phantomjs-ubuntu.gz https://github.com/mirraj2/PhantomjsUbuntu/blob/master/phantomjs-ubuntu.gz?raw=true
gunzip phantomjs-ubuntu.gz
sudo mv phantomjs-ubuntu /usr/bin/phantomjs
chmod 755 /usr/bin/phantomjs

###############################################################################
# Python module	
# `wheel` for packaging; `virtualenv` for testing

sudo apt-get install python python-dev python-pip --force-yes --assume-yes --fix-broken

# Uncomment out the following line to install multiple versions of Python
# sudo apt-get install python2.7-dev python3.0-dev python3.1-dev python3.2-dev

sudo pip install wheel virtualenv

###############################################################################
# R module
# `roxygen2` for packaging; `svUnit`, `XML` & `libxml2-dev` for testing

sudo apt-get install r-base r-base-dev libxml2-dev --force-yes --assume-yes --fix-broken
sudo Rscript -e "install.packages(c('roxygen2','svUnit','XML'),lib='/usr/lib/R/library',repos='http://cran.us.r-project.org')"
