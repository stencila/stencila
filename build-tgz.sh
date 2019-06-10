#!/bin/bash

set -e

ARCHIVE_NAME=stencila-deps.tgz

mkdir -p stencila-deps
cp -R node_modules/@stencila/encoda/src stencila-deps/ 
cp -R node_modules/@stencila/encoda/vendor stencila-deps/ 

mkdir -p stencila-deps/node_modules/puppeteer/
cp -R node_modules/puppeteer/.local-chromium stencila-deps/node_modules/puppeteer/

mkdir -p stencila-deps/node_modules/opn
cp -R node_modules/opn/xdg-open stencila-deps/node_modules/opn/

tar czf ${ARCHIVE_NAME} stencila-deps

rm -r stencila-deps
