# ⌨️ Stencila CLI

**Use Stencila in a terminal console on your own machine**

## 📦 Install

The CLI is is early stages of development (all contributions welcome!). We don't recommend installing it yet, but if you are an early adopter 💖, we'd also appreciate any feedback. You can download standalone binaries for MacOS, Windows or Linux from the [latest release](https://github.com/stencila/stencila/releases/latest).

### Windows

To install the latest release download `stencila-<version>-x86_64-pc-windows-msvc.zip` from the [latest release](https://github.com/stencila/stencila/releases/latest) and place it somewhere on your `PATH`.

### MacOS

To install the latest release in `/usr/local/bin` just use,

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash
```

To install a specific version, append `-s vX.X.X`. Or, if you'd prefer to do it manually, download `stencila-<version>-x86_64-apple-darwin.tar.gz` from the one of the [releases](https://github.com/stencila/stencila/releases) and then,

```bash
tar xvf stencila-*.tar.gz
sudo mv -f stencila /usr/local/bin # or wherever you prefer
```

### Linux

To install the latest release in `~/.local/bin/` just use,

```bash
curl -L https://raw.githubusercontent.com/stencila/stencila/master/install.sh | bash
```

To install a specific version, append `-s vX.X.X`. Or, if you'd prefer to do it manually, download `stencila-<version>-x86_64-unknown-linux-gnu.tar.gz` from the one of the [releases](https://github.com/stencila/stencila/releases) and then,

```bash
tar xvf stencila-*.tar.gz
mv -f stencila ~/.local/bin/ # or wherever you prefer
```

## 🚀 Use

Get started by consulting the built in help:

```sh
stencila help
```

## 🛠️ Develop

The CLI is based on the [Rust library](../rust) so you'll need to have [Rust installed](https://rustup.rs) first. Then, get started by cloning this repository and building the CLI binary:

```sh
git clone git@github.com:stencila/stencila
cd stencila/cli
make build
```

If you are contributing code please run formatting and linting checks before submitting PRs:

```sh
make format lint
```
