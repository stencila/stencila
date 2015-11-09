#!/usr/bin/env bash

# Shell script for provisioning Ubuntu 14.02 with the tools
# required to build Stencila library modules
# Currently, this is somewhat minimalitstic and not all make recipes will work. 
# e.g. it does not include installation of nodejs and phantomjs for running `make js-tests`

###############################################################################
# Add package repositories

# C++: add PPA for g++4.8 (default is gcc 4.6 since this is Ubuntu 12.04)
sudo add-apt-repository ppa:ubuntu-toolchain-r/test -y

# R: add CRAN repository for most recent version
sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E084DAB9
sudo add-apt-repository "deb http://cran.us.r-project.org/bin/linux/ubuntu precise/"

sudo apt-get update

###############################################################################
# C++ module

sudo apt-get install g++-4.8 --force-yes --assume-yes --fix-broken
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-4.8 50
sudo apt-get install build-essential cmake git g++ libssl-dev dos2unix --force-yes --assume-yes --fix-broken

###############################################################################
# Python module	
# `wheel` for packaging; `virtualenv` for testing

sudo apt-get install python python-dev python-pip --force-yes --assume-yes --fix-broken
sudo pip install wheel virtualenv

###############################################################################
# R module
# `Rcpp` for compilation; `roxygen2` for packaging; `svUnit`, `XML` & `libxml2-dev` for testing

sudo apt-get install r-base r-base-dev libxml2-dev --force-yes --assume-yes --fix-broken
sudo Rscript -e "install.packages(c('Rcpp','roxygen2','svUnit','XML'),lib='/usr/lib/R/library',repos='http://cran.us.r-project.org')"
