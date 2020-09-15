# Developing with Docker

If you don't want to install npm on your host directly, you can build the provided
Dockerfile:

```bash
$ docker build -t stencila/schema .
```

And then bind your local schemas directory and run commands from the container.
Here is how we would run `make docs`

```bash
# Run an interactive shell with your schemas bound
$ docker run --rm -it -v $PWD/schemas:/code/schemas stencila/schema bash
$ make docs
```
