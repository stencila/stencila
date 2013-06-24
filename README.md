![Stencila](http://static.stenci.la/img/logo-name-400x88.png)
 
##About
 
[Stencila](http://stenci.la) is a platform for doing stuff with data. 
This is an open source library for the platform. 
We're too busy coding to write much more about it than that. 
But, as the library matures, what we mean by "doing stuff" should become clear.

##Documentation

Various forms of [documentation are available](http://docs.stenci.la).
The auto-generated [documentation for the C++ library](http://docs.stenci.la/cpp/) is currently the most up to date.
The [user guide](http://docs.stenci.la/guide/intro.html) is still in a very early draft.

##Installation

Right now the Stencila library is in a very preliminary state.
We don't recommend actually using it quite yet!
But, if you really, really want to, then...

### C++ library

The Stencila C++ library is available for [download](http://get.stenci.la/cpp/stencila.cpp.tar.gz):

```sh
wget http://get.stenci.la/cpp/stencila.cpp.tar.gz | tar xvf
```

### R package

The Stencila R package is available from our R package repository:

```r
install.packages('stencila',repo='http://get.stenci.la/r')
'''

## Building

On Linux, if you have the normal build tools like g++ and make install, then building the Stencila library should be
fairly easy. For a start, try:

```sh
make all
```

On Windows, our Makefiles are configured to work using [MSYS](http://www.mingw.org/wiki/MSYS). We have written some [instructions for setting up a 
MSYS environment](https://github.com/stencila/stencila/tree/master/building-on-windows.md) suitable for building Stencila.

## Testing

The Stencila [continuous integration server](http://ci.stenci.la) builds packages and run tests when commits are made to this repository.
See the C++ job for [test results](http://ci.stenci.la/job/stencila.cpp/lastCompletedBuild/testReport/%28root%29/) and [coverage reports](http://ci.stenci.la/job/stencila.cpp/lastCompletedBuild/cobertura/stencila/).

##Versioning

It is still early days so the API will change frequently. 
We are using [sematic version numbers](http://semver.org/) so versions like "0.y.z" indicate that the library is still in initial develpment phase. 
Don't rely on API stability until the release of version 1.0.0.

##Licence

Stencila is [ISC Licenced](http://en.wikipedia.org/wiki/ISC_license):

	Copyright (c) 2012 Stencila Ltd

	Permission to use, copy, modify, and/or distribute this software for any purpose with
	or without fee is hereby granted, provided that the above copyright notice and this
	permission notice appear in all copies.

	THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
	WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
	AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT,
	OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE,
	DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
	ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
