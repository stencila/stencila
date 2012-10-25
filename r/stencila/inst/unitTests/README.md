# Unit tests for the Stencila R package

## Packages used

Currently unit tests are written in the `RUnit` format.
This allows us to use both `RUnit` and `svUnit` (which allows
for reporting via the Jenkins based [Stencila Continuous Integration Server](http://ci.stenci.la)).


## Unit test files

There is a separate test file for each Stencila class with
the name `runit.<class>.R`.

The file `runit.checks.R` simple contains examples of the usage of
the `check` family of functions used by `Runit` and `svUnit`.

## Usage

Run either `do.RUnit.R` or `do.svUnit.R`.
Those scripts output to `Runit.txt` and `svUnit.xml` respectively.
