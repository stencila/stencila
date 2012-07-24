# Stencila documentation

## Build

To build all the documentation: 

```sh
make build
```

## Deploy

The documentation is served at `docs.stenci.la` using [Github Pages](http://pages.github.com).
That is what the `CNAME` file is for.
There is a `gh-pages` branch that is updated and pushed using the [ghp-import](https://github.com/davisp/ghp-import)
by doing:

```sh
make deploy
```

