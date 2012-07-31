# Stencila R package

## Requirements

Rcpp
roxygen2

## Build strategy

Stencila C++, and thus Stencila R, relies on numerous open source packages.
Rather than distributeing the source and hoping that the user will have all the necessary dependencies to do an install, we
compile shared libraries (.so and .dll) files and distribute those instead.
