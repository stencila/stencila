FROM stencila/ubuntu-14.04

# Install Python
RUN apt-get install -y python2.7 python2.7-dev

# Do following install tasks in /tmp
WORKDIR /tmp

# Install latest pip
RUN wget https://bootstrap.pypa.io/get-pip.py
RUN python2.7 get-pip.py

# Install Stencila Python package
RUN pip install http://get.stenci.la/py/stencila-0.20-cp27-none-linux_x86_64.whl

# Install other Python modules
RUN apt-get install -y build-essential pkg-config libpng-dev libfreetype6-dev
RUN pip install matplotlib

# Add a user (creating home and specifying shell) and
# use it when running the container
RUN useradd --create-home --shell /bin/bash stencila
USER stencila
WORKDIR /home/stencila

# Add a Stencila storage directory
RUN mkdir -p /home/stencila/store
ENV STENCILA_STORES /home/stencila/store
