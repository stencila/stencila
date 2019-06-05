# Stencila

> :sparkles: This is the `next` branch where we are doing major refactoring towards treating this repo as a top level 'umbrella' package and main entry point for off-line users. We will deprecate `stencila/cli` and `stencila/desktop` and instead build those binaries here.

[![NPM](http://img.shields.io/npm/v/stencila.svg?style=flat)](https://www.npmjs.com/package/stencila)
[![Build status](https://travis-ci.org/stencila/stencila.svg?branch=master)](https://travis-ci.org/stencila/stencila)
[![Code coverage](https://codecov.io/gh/stencila/stencila/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/stencila)

### Development

```bash
npm install
npx ts-node-dev src/cli serve --sync
```

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
