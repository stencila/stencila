# stencila.sqlite : Functions for working with Stencila datasets within SQLite

## Motivation

SQLite is a fast and reliable database engine.
Unfortuntely, the base SQLite engine comes with a limited number of [core](http://www.sqlite.org/lang_corefunc.html), [date/time](http://www.sqlite.org/lang_datefunc.html) and [aggregate](http://www.sqlite.org/lang_aggfunc.html) functions.
`stencila.sqlite` provides a range of mathematical, statistical and other functions for doing interesting things with your data within SQLite.

## Usage

Within the `sqlite3` console simply load the library,

```
SELECT load_extension('./stencila.so');
```

and start using using the functions:

```
SELECT stencila_version();
SELECT sqrt(sd(cos(x))) FROM my_table;
```
## Development

This library makes use of the SQLite C API to extend its functionality.
SQLite allows for [loadable extensions](http://www.sqlite.org/cvstrac/wiki?p=LoadableExtensions) which can be loaded at runtime.

SQLite extensions can be of three types:

* Functions e.g. length(), strftime()
* Aggegators e.g. sum(), avg()
* Collations (used in ODER BY)

Currently, `stencila.sqlite` provides a limited set of math functions

* Math: acos, asin, atan, atan2,degrees, radians, cos, sin, tan, cot, cosh, sinh, tanh, coth, exp,
log, log10, power, sign, sqrt, square, ceil, floor, pi.


* Aggregate: mean, geomean, harmean, sd, variance

Our intension is to add more functions over time by levering existing C/C++ libaries

## Building

Building this library requires the sqlite3 header files (on Ubuntu this is available from the sqlite3-dev package).
There is a rudimentary Makefile for the project so assuming you have the usual make tools installed:

```
make
```

