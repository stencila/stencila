FROM stencila/ubuntu-14.04

# Turn off interactive installs
# See https://github.com/phusion/baseimage-docker/issues/58
ENV DEBIAN_FRONTEND noninteractive

# Update system and install required system packages
RUN apt-get update
RUN apt-get install -y wget software-properties-common

# Install R
RUN add-apt-repository 'deb http://cran.us.r-project.org/bin/linux/ubuntu trusty/'
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E084DAB9
RUN apt-get update
RUN apt-get install -y r-base=3.2.2-1trusty0 r-base-dev=3.2.2-1trusty0 r-recommended=3.2.2-1trusty0

# Do following install tasks in /tmp
WORKDIR /tmp

# Install Stencila R package dependencies
RUN Rscript -e "install.packages('Rcpp',repo='http://cran.cnr.berkeley.edu/')"

# Install Stencila R package using tar so that version can be specified
# RUN gets cached is just using `install.packages('stencila',repo='http://get.stenci.la/r')`
RUN wget http://get.stenci.la/r/src/contrib/stencila_0.19.tar.gz ; \
    R CMD INSTALL stencila_0.19.tar.gz

# Add Stencila CLI script to PATH
RUN Rscript -e 'require(stencila); stencila:::install()'

# Add a user (creating home and specifying shell) and
# use it when running the container
RUN useradd --create-home --shell /bin/bash stencila
USER stencila
WORKDIR /home/stencila

# Add a Stencila storage directory
RUN mkdir -p /home/stencila/store
ENV STENCILA_STORES /home/stencila/store
