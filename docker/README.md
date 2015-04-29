![Stencila](http://static.stenci.la/img/logo-name-400x88.png)

Docker containers for using Stencila components.

# Images

Images are built using [Automated Builds on Docker Hub](https://docs.docker.com/docker-hub/builds/) from the `Dockerfile`s in this repository. Currently, the available images are:

- [`stencila/ubuntu-14.04-python-2.7`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-python-2.7/)
- [`stencila/ubuntu-14.04-r-3.1`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-r-3.1/)

# Building and testing

During development you might build and test an image like this before pushing and triggering an automated build

```sh
cd ubuntu-14.04-r-3.1
sudo docker build --tag stencila/ubuntu-14.04-r-3.1 .
sudo docker run --interactive --tty stencila/ubuntu-14.04-r-3.1 /bin/bash
sudo docker run --detach --publish=7373:7373 stencila/ubuntu-14.04-r-3.1 stencila-r core/stencils/examples/kitchensink serve:Inf
```

# Vagrant

There is a `Vagrantfile` for launching a [Vagrant](https://www.vagrantup.com/) virtual machine provisioned with Docker. It can be useful for building and testing Docker images.

Start a VM,

```sh
vagrant up
```

SSH into it and change into this directory,
 
```sh
vagrant ssh
cd /vagrant
```

and build and use an image (as above). When finished shutdown the VM with `halt`:

```sh
vagrant halt
```
