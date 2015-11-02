# Docker containers for using Stencila components.

Images are built using [Automated Builds on Docker Hub](https://docs.docker.com/docker-hub/builds/) from the `Dockerfile`s in this repository. Currently, the available images are:

- [`stencila/ubuntu-14.04-python-2.7`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-python-2.7/)
- [`stencila/ubuntu-14.04-r-3.2`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-r-3.2/)


Run an image with a Stencila session for a component like this:

```sh
sudo docker run --detach --publish=7373:7373 stencila/ubuntu-14.04-r-3.2 stencila-r core/stencils/examples/kitchensink serve ...
```

The first time you do this may be slow because the image will need to be pulled from the Docker hub. One the Docker image is running go to [http://localhost:7373/core/stencils/examples/kitchensink](). This might also be a little slow the frst time because the Docker instance needs to `git clone` the kitchensink first.

When done kill the docker instance

```sh
sudo docker ps
sudo docker kill bc72075ae56d
```

# Building and testing

You should build and test an image like this before pushing and triggering an automated build:

```sh
cd ubuntu-14.04-r-3.2

# Build an image
sudo docker build --tag stencila/ubuntu-14.04-r-3.2 .

# Test it interactively
sudo docker run --interactive --tty stencila/ubuntu-14.04-r-3.2 /bin/bash

# Launch a Stencila R session with the kitchen sink example
sudo docker run --detach --publish=7373:7373 stencila/ubuntu-14.04-r-3.2 stencila-r core/stencils/examples/kitchensink serve ...
open http://localhost:7373/core/stencils/examples/kitchensink

# When done kill the docker instance
sudo docker ps
sudo docker kill bc72075ae56d
```
