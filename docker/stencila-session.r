#!/usr/bin/env Rscript

# An R script for running a Stencila session in a Docker container

library(stencila)
stencila:::serve()
Sys.sleep(Inf)
