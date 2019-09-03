#!/bin/bash

set -e

ARCHIVE_NAME=stencila-deps.tgz

mkdir -p stencila-deps

cp --parents --recursive node_modules/@stencila/encoda/dist/codecs/pandoc/templates stencila-deps
cp --parents --recursive node_modules/@stencila/encoda/vendor stencila-deps
cp --parents --recursive node_modules/puppeteer/.local-chromium stencila-deps
cp --parents --recursive node_modules/opn/xdg-open stencila-deps

tar czf ${ARCHIVE_NAME} stencila-deps

rm -r stencila-deps
