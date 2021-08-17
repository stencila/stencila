# Usual calls
library(pkg1)
require(pkg2, verbose = TRUE)

# Using the `character.only` option
library("pkg3", character.only = TRUE, verbose = TRUE)

# A library call inside a function
foo <- function () {
    library(pkg4)
}

# Imports that are not detected

library(paste("some_pkg", "_name"))

pkg_var <- "some_package_name"
library(pkg_var, character.only = TRUE)
