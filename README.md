<div align="center">
	<img src="https://stenci.la/img/stencila/stencilaLogo.svg" alt="Stencila" style="max-width:300px">
</div>
<br>

> ⚠️ This repo is currently undergoing a bit of a reboot. Please bear with us while we get it organized.

> 🦄 We are using using [README Driven Development](https://tom.preston-werner.com/2010/08/23/readme-driven-development.html) in this repository. That means, depending on when you read this, some (a lot!) of the features described below don't actually exist yet. As far as possible, we indicate non-existing features using a unicorn emoji with either square brackets e.g. 🦄 [a cool feature that's not yet implemented], or a link to an issue about the feature e.g. 🦄 [a link to the issue for the feature](https://github.com/stencila/stencila/issues).

## 🎁 Hub

If you don't want to install anything and/or just want to try out Stencila https://hub.stenci.la is the best place to start.

## 👩‍💻 Language packages

If you prefer, you can use Stencila from within your favourite programming lanaguage. The following `stencila` packages for each language 🦄[include the same functionality as the CLI] but accessible via functions e.g. `convert`, `execute` etc

### JavaScript / TypeScript

The `stencila` Node.js package is available from NPM,

```sh
npm install stencila
```

### Python

The `stencila` Python package 🦄 [is available from PyPI],

```sh
python3 -m pip install stencila
```

### R

The `stencila` R package 🦄 [is available from CRAN]. To install it from within R,

```r
install.packages("stencila")
```

Or, from the command line,

```sh
Rscript -e 'install.packages("stencila")'
```

The R package 🦄 [includes an RStudio Add-in] that makes it even easier to get started using Stencila with R.

### Rust

The `stencila` Rust package 🦄 [is available via crates.io],

```sh
cargo add stencila
```

### Other

Is your favourite language missing from the above list? [Let us know!](https://github.com/stencila/stencila/discussions/new)

<br>

## 🔌 Plugins

On their own, the desktop application, CLI tool and language packages provide little functionality. Instead they rely on _plugins_ to provide much of their functionality.

You can 🦄 [install plugins] using the `stencila` CLI tool using it's name or an alias,

```sh
stencila plugins install <name or alias>
```

The following table lists the main plugins. These plugins are in various stages of development and not all of them are compatible with the CLI. Generally, it won't be worth installing them prior to `v1` and coverage of at least 90%.

| Plugin   | Aliases              | Version     | Coverage    | Primary functionality                                   |
| -------- | -------------------- | ----------- | ----------- | ------------------------------------------------------- |
| [encoda] | `converter`          | ![encoda-v] | ![encoda-c] | Convert stencils between file formats                   |
| [jesta]  | `node`, `javascript` | ![jesta-v]  | ![jesta-c]  | Compile, build and execute stencils that use JavaScript |
| [rasta]  | `r`                  | ![rasta-v]  | ![rasta-c]  | Compile, build and execute stencils that use R          |
| [pyla]   | `python`             | ![pyla-v]   | ![pyla-c]   | Compile, build and execute stencils that use Python     |
| [jupita] | `jupyter`            | ![jupita-v] | ![jupita-c] | Execute stencils using Jupyter kernels                  |
| [dockta] | `docker`             | ![dockta-v] | ![dockta-c] | Build Docker images for stencils                        |
| [nixta]  | `nix`                | ![nixta-v]  | ![nixta-c]  | Build Nix environments for stencils                     |

[encoda]: https://github.com/stencila/encoda#readme
[jesta]: https://github.com/stencila/jesta#readme
[pyla]: https://github.com/stencila/pyla#readme
[rasta]: https://github.com/stencila/rasta#readme
[jupita]: https://github.com/stencila/jupita#readme
[dockta]: https://github.com/stencila/dockta#readme
[nixta]: https://github.com/stencila/nixta#readme
[encoda-v]: https://img.shields.io/github/v/release/stencila/encoda?label=
[jesta-v]: https://img.shields.io/github/v/release/stencila/jesta?label=
[rasta-v]: https://img.shields.io/github/v/release/stencila/rasta?label=
[pyla-v]: https://img.shields.io/github/v/release/stencila/pyla?label=
[dockta-v]: https://img.shields.io/github/v/release/stencila/dockta?label=
[nixta-v]: https://img.shields.io/github/v/release/stencila/nixta?label=
[jupita-v]: https://img.shields.io/github/v/release/stencila/jupita?label=
[encoda-c]: https://img.shields.io/codecov/c/github/stencila/encoda?label=
[jesta-c]: https://img.shields.io/codecov/c/github/stencila/jesta?label=
[rasta-c]: https://img.shields.io/codecov/c/github/stencila/rasta?label=
[pyla-c]: https://img.shields.io/codecov/c/github/stencila/pyla?label=
[dockta-c]: https://img.shields.io/codecov/c/github/stencila/dockta?label=
[nixta-c]: https://img.shields.io/codecov/c/github/stencila/nixta?label=
[jupita-c]: https://img.shields.io/codecov/c/github/stencila/jupita?label=
