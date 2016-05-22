# Docker containers for using Stencila components.

Images are built using the `Dockerfiles` in this directory. Currently, the available images are:

- [`stencila/ubuntu-14.04-r-3.2`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-r-3.2/)
- [`stencila/ubuntu-14.04-py-2.7`](https://registry.hub.docker.com/u/stencila/ubuntu-14.04-py-2.7/)

Run a docker container with one of these images, binding port 7373:

```sh
docker run --publish=7373:7373 stencila/ubuntu-14.04-r-3.2 stencila-session
```

The first time you do this may be slow because the image will need to be pulled from the Docker hub. This command will launch a session in the host language (in this example R), import the Stencila package and start the embedded HTTP/Websocket server. To access a component (e.g. a sheet), open the component's URL in your browser:

```
http://localhost:7373/demo/sheets/simple-r/
```

This might also be a little slow the first time because the Docker instance needs to `git clone` the component into the Docker container before serving it.

You may have to use an IP address instead of `localhost` to access the Docker container. On Mac OSX (and probably Windows too?) you need to forward the 7373 port from the Docker VirtualBox VM through to host machine like this:

```sh
VBoxManage modifyvm "default" --natpf1 "tcp-port-7373,tcp,,7373,,7373";
VBoxManage modifyvm "default" --natpf1 "udp-port-7373,udp,,7373,,7373";
```

When done kill the docker instance

```sh
sudo docker ps
sudo docker kill <id-of-instance>
```

# Debugging

If you want to see a bit more of what's going on inside the R session inside the Docker container you can start the container in interactive mode:

```
docker run --publish=7373:7373 --interactive --tty stencila/ubuntu-14.04-r-3.2 bash
```

Then inside the container, at the bash prompt, start a session in the background and tail the embedded server's logs:

```
stencila-session &
tail -f /tmp/stencila/logs/server-*.log
```

# Building and testing

You should build and test an image like this before pushing:

```sh
cd ubuntu-14.04-r-3.2

# Build an image
sudo docker build --tag stencila/ubuntu-14.04-r-3.2 .

# Test it interactively
sudo docker run --interactive --tty stencila/ubuntu-14.04-r-3.2 bash

# Launch a Stencila R session and open a demo component
sudo docker run --publish=7373:7373 stencila/ubuntu-14.04-r-3.2 stencila-session
open http://localhost:7373/demo/stencils/ggplot2-diamonds/

# You can run a shell in that container to debug issues... 
sudo docker ps
docker exec --interactive --tty bc72075ae56d bash

# When done kill the docker instance
sudo docker kill bc72075ae56d
```
