# Docker containers for using Stencila components.

Images are built using [Automated Builds on Docker Hub](https://docs.docker.com/docker-hub/builds/) from the `Dockerfiles` in this repository. Currently, the available images are:

- [`stencila/ubuntu-14.04-python-2.7`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-python-2.7/)
- [`stencila/ubuntu-14.04-r-3.2`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-r-3.2/)

Check those links for current image build status. 

Run an image with a Stencila session for a component like this:

```sh
sudo docker run --detach --publish=7373:7373 stencila/ubuntu-14.04-r-3.2 stencila-r demo/stencils/kitchen-sink serve ...
```

The first time you do this may be slow because the image will need to be pulled from the Docker hub. Once the Docker image is running go to http://localhost:7373/demo/stencils/kitchen-sink. This might also be a little slow the first time because the Docker instance needs to `git clone` the stencil first.

On Mac OSX (and probably Windows too?) you need to forward the 7373 port from the Docker VirtualBox VM through to host machine like this:

```sh
VBoxManage modifyvm "default" --natpf1 "tcp-port-7373,tcp,,7373,,7373";
VBoxManage modifyvm "default" --natpf1 "udp-port-7373,udp,,7373,,7373";
```

When done kill the docker instance

```sh
sudo docker ps
sudo docker kill <id-of-instance>
```

# Building and testing

You should build and test an image like this before pushing and triggering an automated build:

```sh
cd ubuntu-14.04-r-3.2

# Build an image
sudo docker build --tag stencila/ubuntu-14.04-r-3.2 .

# Test it interactively
sudo docker run --interactive --tty stencila/ubuntu-14.04-r-3.2 bash

# Launch a Stencila R session with the kitchen sink example
sudo docker run --detach --publish=7373:7373 stencila/ubuntu-14.04-r-3.2 stencila-r demo/stencils/kitchen-sink serve ...
open http://localhost:7373/demo/stencils/kitchen-sink

# You can run a shell in that container to debug issues... 
sudo docker ps
docker exec --interactive --tty bc72075ae56d bash

# When done kill the docker instance
sudo docker kill bc72075ae56d
```
