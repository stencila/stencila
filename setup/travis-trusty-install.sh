#!/bin/bash

sudo apt-get -yq remove r-base r-base-core
sudo apt-get -yq --no-install-suggests --no-install-recommends --force-yes \
    install r-base=3.2.5-1trusty0 r-base-core=3.2.5-1trusty0 r-recommended=3.2.5-1trusty0 r-base-dev=3.2.5-1trusty0
 