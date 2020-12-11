# Stencila

Welcome to the main entry point to the Stencila ecosystem. This repo contains the `stencila` command line tool which is the top level 'umbrella' package to use our various libraries.

> :sparkles: We are undergoing a major reboot in this repository (and many of our others) to build a single entry point for reproducible documents. We will deprecate `stencila/cli` and `stencila/desktop` and instead build those binaries here. We will also be deprecating binary builds of our other repositories as they are integrated into this one.

[![NPM](http://img.shields.io/npm/v/stencila.svg?style=flat)](https://www.npmjs.com/package/stencila)

# Install

Stencila is available as a pre-compiled, standalone command line tool.

### Windows

To install the latest release of the `stencila` command line tool, download `stencila-win-x64.zip` for the [latest release](https://github.com/stencila/stencila/releases/) and place it somewhere on your `PATH`.

### MacOS

To install the latest release of the `stencila` command line tool to `/usr/local/bin` just use,

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash
```

To install a specific version, append `-s vX.X.X` e.g.

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash -s v0.33.0
```

Or, if you'd prefer to do things manually, download `stencila-macos-x64.tar.gz` for the [latest release](https://github.com/stencila/stencila/releases/) and then,

```bash
tar xvf stencila-macos-x64.tar.gz
sudo mv -f stencila /usr/local/bin # or wherever you like
```

### Linux

To install the latest release of the `stencila` command line tool to `~/.local/bin/` just use,

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash
```

To install a specific version, append `-s vX.X.X` e.g.

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash -s v0.33.0
```

Or, if you'd prefer to do things manually, or place stencila elsewhere, download `stencila-linux-x64.tar.gz` for the [latest release](https://github.com/stencila/stencila/releases/) and then,

```bash
tar xvf stencila-linux-x64.tar.gz
mv -f stencila ~/.local/bin/ # or wherever you like
```
