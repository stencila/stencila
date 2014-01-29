# Define the Stencila directory path. Done by obtaining the directory of this
# Makefile, not by using $(shell pwd) which depends upon where make is called from
STENCILA_HOME := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))

# Define a platform variable. 
# This can be overidden by passing the variable from the command line e.g. make STENCILA_PLATFORM=win all
# See alternatives for doing this
#   http://stackoverflow.com/questions/714100/os-detecting-makefile
#   http://stackoverflow.com/questions/4058840/makefile-that-distincts-between-windows-and-unix-like-systems
# The name STENCILA_PLATFORM is used because OS may be already defined as an environment variable
ifndef STENCILA_PLATFORM
	ifeq ($(shell uname -o), GNU/Linux)
		STENCILA_PLATFORM := linux
	endif
	ifeq ($(shell uname -o), Msys)
		STENCILA_PLATFORM := msys
	endif
	ifndef STENCILA_PLATFORM
		$(error Unhandled operating system $(shell uname -o))
	endif
endif

# Define variables for compilation tools using the standard make implicit variables 
#  http://www.gnu.org/software/make/manual/html_node/Implicit-Variables.html
ifeq ($(STENCILA_PLATFORM), linux)
	CC := gcc
	CXX := g++
	AR := ar
endif
ifeq ($(STENCILA_PLATFORM), msys)
	CC := gcc
	CXX := g++
	AR := ar
endif

# Get Stencila library version number
# This uses "--long" so that git produces the same format output each time (even just after a new tag)
# The sed command is based on http://www.linuxquestions.org/questions/linux-general-1/parsing-a-version-number-with-regular-expressions-737095/
STENCILA_VERSION :=  $(shell git describe --long | sed -r 's/([0-9]+)\.([0-9]+)(-([0-9]+)-g[0-9A-Fa-f]*)?/\1.\2.\4/g')
