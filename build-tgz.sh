#!/bin/bash

set -e

ARCHIVE_NAME=stencila-deps.tgz

mkdir -p stencila-deps

if [[ "$(uname)" == "Darwin" ]]; then
  CP="rsync -R"
else
  CP="cp --parent --recursive"
fi

$CP node_modules/@stencila/encoda/dist/codecs/pandoc/templates stencila-deps
$CP node_modules/@stencila/encoda/vendor stencila-deps
$CP node_modules/puppeteer/.local-chromium stencila-deps
$CP node_modules/opn/xdg-open stencila-deps

tar czf ${ARCHIVE_NAME} stencila-deps

rm -r stencila-deps
