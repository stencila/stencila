# Stencila `.Rprofile`s example

This example workspace is for testing that Stencila will load the closest `.Rprofile` file (close in terms of distance up the directory tree) when starting a R microkernel.

This is used in a test in the `rust/kernel-r` crate. But you can also test it out by opening the `*.smd` files in each of the directories, running the R code and checking that the outputted value is as expected.

See https://github.com/stencila/stencila/issues/2389 for why this behavior is useful.
