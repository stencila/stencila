#!/bin/bash

# A script for installing system requirements on OS X
# Avoids the need for lots of "if [[ "$TRAVIS_OS_NAME" == "osx" ]]; ..." statements
# in the `.travis.yml` and easier to read for others wondering how to setup
# for a build on their own OS X system

brew update

# GNU sed because OSX sed is old and does not have same behaviour as Linux sed
# as expected in the Makefile
brew install --default-names gnu-sed

# Newer version of OpenSSL
# Env vars need to be set to prevent linker error
# See https://github.com/rust-lang/cargo/issues/2295#issuecomment-173144879
brew install openssl
brew link --force openssl
export OPENSSL_ROOT_DIR=$(brew --prefix openssl)
export OPENSSL_LIB_DIR=$(brew --prefix openssl)"/lib"
export OPENSSL_INCLUDE_DIR=$(brew --prefix openssl)"/include"

# R
brew tap homebrew/science
brew install R
Rscript -e "install.packages(c('Rcpp','codetools','roxygen2','svUnit'),repo='http://cloud.r-project.org/')"

# Lemon parser (Flex already installed)
brew install lemon

