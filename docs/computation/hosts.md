# Hosts

In Stencila, a `Host` provides execution `Environments` and, within those environments, execution `Contexts`. There are several implementations of `Hosts` including in the individual language packages (e.g. `stencila/r`, `stencila/py`) and `stencila/hub`.

## `/environs`

The `/environs` endpoint lists the execution `Environments` supported by the host. When the Stencila client (e.g Stencila Desktop) connects to a host if first visits this endpoint to see what environments are available to execute a project within.

When you run a Stencila `Host` locally, it returns a single "local" environment. For example, you can run the `Host` in the `stencila/node` package:

```
npm install stencila/node -g
stencila-node
```

and then visit `http://localhost:2000/environs`:

```json
$ http localhost:2000/environs
[
  {
    "id": "local", 
    "name": "local",
    "version": null,
    "servers": {
      "http": {
        "path": "/"
      }
    }
  }
]
```

The property `id` is an environment identifier, made up of components `name` and `version`. The `servers` property indicates the ways that you can connect to the environment . Currently only a `http` protocol is available, but in the future other, additional, protocols e.g. Websocket may be implemented. The `path` property indicates the relative path to connect to the environment - in this case it's `/` because we'll be connecting o the _same_ environment. 

The `local` environment `id` just signifies that you will be executing the code within your project within your local machine environment - which has whatever packages and libraries you have installed. That's not easily reproducible, so let's run a host within a better defined execution environment. The `stencila/images` repository defines several execution environments using the Nix package manager. These environments are available as Docker images which you can run on your own machine:

```bash
docker run --rm -it -p 2100:2000 stencila/core:0.28
```

and then visit `http://localhost:2100/environs`:

```bash
$ http localhost:2100/environs
[
  {
    "id": "stencila/core@0.28", 
    "name": "stencila/core",
    "version": "0.28", 
    "servers": {
      "http": {
        "path": "/"
      }
    }
  }
]

```

There is only one environment in the list, but it's `id` is `stencila/core@0.28`, instead of `local`.

> The `stencila/core` Docker image has several Stencila language packages installed (currently `stencila/node`, `stencila/py` and `stencila/r`). The `stencila/node` `Host` acts as the 'gateway' into the image and passes on requests for certain execution contexts e.g. `RContext`, `PythonContext` to the other hosts. The `stencila/node` `Host` 'knows' that it is within the `stencila/core` environment because the `STENCILA_ENVIRON` environment variable is set to `stencila/core@0.28` in that image.

> This example also illustrates why some `Hosts` provide a `server.http.path`: the host inside the Docker container does not "know" that it is only accessible at `http://localhost:2100`.


Some `Hosts` provide more than one environment. For example, `stencila/cloud` provides access to several execution environments including `stencila/core`, `stencila/mega` etc. `Hosts` can also redirect you to environments located at other URLs and provide the access tokens needed to connect to them e.g.


```
[
  {
    "id": "stencila/core@0.28", 
    "name": "stencila/core",
    "version": "0.28", 
    "servers": {
      "http": {
        "url": "https://cluster1.my-uni.edu/core@0.28"
        "token": "518f4781d9a9b139d6999557eb0"
      }
    }
  },{
    "id": "stencila/mega@0.28", 
    "name": "stencila/mega",
    ...
]

```

## Authentication

The `/environs` endpoint can be accessed freely but all other endpoints of the host usually require authentication. This prevents a malicious user from executing arbitrary code in the execution context.


## `/manifest`

The `/manifest` endpoint provides a manifest for the host detailing aspect such as types of execution contexts that it supports. e.g.

```bash
$ http :2000/manifest
{
  "id": "node-8536c01a9f271bca4b8e85c06b77738c4cca54ee7f45ba06", 
  "stencila": {
    "package": "node", 
    "version": "0.28.4"
  },
  "process": {
    "arch": "x64", 
    "name": "node", 
    "pid": 18741, 
    "platform": "linux", 
    "version": "v8.9.1"
  }, 
  "run": [
    "/usr/bin/node", 
    "-e", 
    "require('stencila-node').run()"
  ], 
  "servers": {
    "http": {
      "token": "02398102d6c1c8d6294bab43", 
      "url": "http://127.0.0.1:2000"
    }
  }, 
  "types": {
      "NodeContext": {
          "client": "ContextHttpClient", 
          "name": "NodeContext"
      }
  },
  ....

```

