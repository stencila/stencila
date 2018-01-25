# Host interface - Iteration I

## Goal

Implement `HostComponent` to allow the user to troubleshoot connections to external `Hosts` and see which `Contexts` are hosted where.

## Setup

To test connections to external hosts it will be useful to run multiple external hosts, of different types, as part of the dev setup.

You can run a Node.js Stencila Host using:

```bash
> npm install stencila-node
> STENCILA_AUTHORIZATION=false ./node_modules/.bin/stencila-node
Host has started at: http://127.0.0.1:2020
```

Connect to this host using the URL displayed e.g. `http://127.0.0.1:2020` (authorization is turned off to make connecting easier)

You can run the `base` Stencila Docker image which runs a Node.js Stencila Host with Python and R Stencila Hosts as peers within the Docker image which are spawned as needed:

```bash
> docker run -it --rm -p 2100:2000 stencila/base
Host has started at: http://0.0.0.0:2000
```

Note that the message printed out when running the Docker image says it's port is `2000`. However, because the publish option binds to `2100` (`-p 2100:2000`) it should be connected to using `http://127.0.0.1:2100`. You can use the `base` or the `core` image for this testing but the `base` image is a lot smaller.

You can prepopulate the internal host's list of peers (i.e. `host.peers`) using the `peers` query parameter in the URL e.g. http://localhost:4000/examples/index.html?example=external-cells&peers=http://127.0.0.1:2000,http://127.0.0.1:2100

## Tasks

- `HostComponent` that can be opened in the side panel (see 0007-hosts-interface-wireframe.svg):

![0007-hosts-interface-wireframe.svg]()

- a list of hosts starting with the _internal_ host, followed by _external_ hosts (get the list from `host.peers`)
- under each host a nested list of contexts that is supports (get the list from `host.peers[].types`)
- highlight the contexts that have been instantiated (those in `host._instances`) and where they are hosts (match the url in `instance.url`) 
- a URL input field so that users can add the URL for an external host that they want to connect to; when updated call `host.pokePeer(url)`
- a checkbox to turn on autodiscovery of localhosts: `host.discover()`
