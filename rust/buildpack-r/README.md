# 📦 Stencila buildpack for R

## Detection

Matches against a project that has in its root folder:

  - a `.tool-versions` file with a `R` entry, or

  - a `renv.lock` file of a `renv` folder, or

  - any of `DESCRIPTION`, `install.R`, `install.r`, `main.R`, `main.r`, `index.R`, or `index.r`.

## R version

The version of Node.js to be installed is determined by (in descending order of precedence):

  - the `R` entry of any `.tool-versions` file,

  - the `R.Version` entry of the `renv.lock` file,

  - the latest version installed on the system,

  - the latest version of R available for download.

## R packages

For R projects, Stencila leans on the excellent [renv](https://rstudio.github.io/renv/index.html) project

  - if an `renv.lock` file is present then `renv::restore()` will be used to install the dependencies specified in it,

  - if an `install.R` file is present then `Rscript` will be used to run that script,

  - otherwise, `renv::snapshot()` will be used to generate a new `renv.lock` file that will then be `renv::restore()`d from.