#!/usr/bin/env bash

# Shell script for provisioning Ubuntu 14.04 with the tools
# required to use Stencila packages

sudo apt-get update

###############################################################################
# Git is currently required for cloning Stencila components
sudo apt-get install git

###############################################################################
# Python package

# Install Python 2.7
sudo apt-get install python2.7 --force-yes --assume-yes --fix-broken

###############################################################################
# R package

# Install R 3.2 from CRAN repository
sudo add-apt-repository "deb http://cran.us.r-project.org/bin/linux/ubuntu trusty/"
sudo apt-get update
sudo apt-get install r-base --force-yes --assume-yes --fix-broken
