#!/usr/bin/env bash

# Shell script for provisioning Ubuntu 14.04 in similar way as on Travis CI
# Useful for trying things out instead of cycles of editing+pushing `../.travis.yml`

export DEBIAN_FRONTEND=noninteractive

# Currently just setting up Python to try and work out why that is failing here:
#   https://travis-ci.org/stencila/stencila/builds/102494415
sudo apt-get install python python-dev --force-yes --assume-yes --fix-broken
curl --silent --show-error --retry 5 https://bootstrap.pypa.io/get-pip.py | sudo python2.7
sudo -H pip install setuptools wheel virtualenv --upgrade
