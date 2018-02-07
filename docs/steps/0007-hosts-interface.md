---
title: Hosts user interface
author:
  - Nokome Bentley
type: Feature
status: Draft
---

## Introduction

In Stencila a `Host` provides execution contexts (e.g. `JsContext`, `SqliteContext`) and other resources (e.g. `FileStorer`; these other resources are currently not being used). Hosts share a common API for instantiating and calling methods on these resources.

The _internal_ host is the singleton instance of the class `Host` (in file `src/host/Host.js`) that is instantiated in the browser (or Electron desktop). A _peer_ is just another _host_ known to the current _host_.

External hosts provide a JSON manifest which includes descriptions of the resources they provide. For example, if you start the `Host` in the [Stencila R package](https://github.com/stencila/r) and then request it's manifest using `curl -H "Accept: application/json" http://127.0.0.1:2000` you get something like:

```json
{
    "id": "r-rjuaniyaxbnt92566t4vmmt8rt10es65tx12jdcgy8jegudmpmw6osaiu6ifsiqi", 
    "instances": null, 
    "process": 4235, 
    "run": ["/usr/bin/R", "--slave", "-e", "stencila:::run(echo=TRUE)"],
    "servers": {
        "http": {
            "ticket": "0udvdSil6nrb", 
            "url": "http://127.0.0.1:2010"
        }
    }, 
    "stencila": {
        "package": "r", 
        "version": "0.28.2"
    }, 
    "types": {
        "FileStorer": {
            "aliases": "file", 
            "base": "Storer", 
            "name": "FileStorer"
        }, 
        "RContext": {
            "client": "ContextHttpClient", 
            "name": "RContext"
        }, 
        "SqliteContext": {
            "client": "ContextHttpClient", 
            "name": "SqliteContext"
        }
    }, 
    "urls": "http://127.0.0.1:2010"
}

```

The _internal_ `Host` has a method `registerPeer()` which records these manifests and uses them to instantiate remote execution contexts.

## Rationale

Currently, there is no user interface for seeing which external hosts are available and which execution contexts reside on which hosts. This makes it difficult for the user to troubleshoot problems with the execution of external code cells.


## Implementation

- [Iteration I](0007-hosts-interface-it1.md)
