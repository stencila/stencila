FROM ubuntu:14.04

# Update system and install required system packages
RUN apt-get update
RUN apt-get install -y wget software-properties-common

# Add "helpers" (programs which provide functionality which has not
# been integrated into the base Stencila C++ library)
# See https://github.com/stencila/stencila/blob/master/Makefile#L318 for
# suggestions for installing these

# PhantomJS for exporting stencils
WORKDIR /usr/local/share
RUN wget https://bitbucket.org/ariya/phantomjs/downloads/phantomjs-1.9.8-linux-x86_64.tar.bz2
RUN tar xjf phantomjs-1.9.8-linux-x86_64.tar.bz2
RUN ln -s /usr/local/share/phantomjs-1.9.8-linux-x86_64/bin/phantomjs /usr/local/share/phantomjs
RUN ln -s /usr/local/share/phantomjs-1.9.8-linux-x86_64/bin/phantomjs /usr/local/bin/phantomjs

# Pandoc for exporting stencils
WORKDIR /tmp
RUN wget https://github.com/jgm/pandoc/releases/download/1.13.2/pandoc-1.13.2-1-amd64.deb
RUN dpkg --force-depends --install pandoc-1.13.2-1-amd64.deb
RUN apt-get install -f -y
RUN apt-get install -y texlive

# Node.js for other helpers below
RUN add-apt-repository -y ppa:chris-lea/node.js
RUN apt-get update
RUN apt-get install -y nodejs

# Sass for compiling themes
RUN npm install -y node-sass -g

# UglifyJS for compiling themes
RUN npm install -y uglify-js -g

