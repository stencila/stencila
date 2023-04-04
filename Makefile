# Welcome to the Stencila root Makefile!
#
# This file has several 'recipes' for development tasks which span
# the different programming languages and tools.
#
# Some of the nice things about `Make` include that it, saves you from having
# to remember various command line incantations, it is ubiquitous and it allows
# you to run severval of these recipes, across several languages, at the same time e.g:
#
#   make format lint docs
#
# This root Makefile delegates to the Makefiles in the language specific directories
# e.g. `rust`. See those for what each recipe actually runs for each language.
#
# Having said all that, if you prefer, you can of course try to remember those incantations or
# copy and paste and run them yourself 😼.

all: format lint test audit cli docs

setup:
	make -C rust setup

format:
	make -C rust format

lint:
	make -C rust lint

test:
	make -C rust test

audit:
	make -C rust audit

cli:
	make -C rust cli

generated:
	make -C rust generated

clean:
	make -C rust clean
