#!/bin/bash

# A script for bundling binaries and other assets into `stencila-deps.tgz`

set -e

mkdir -p stencila-deps

if [[ "$(uname)" == "Darwin" ]]; then
  CP="rsync -Ra"
else
  CP="cp --parent --recursive"
fi

$CP node_modules/@stencila/encoda/dist/codecs/csl/styles stencila-deps
$CP node_modules/@stencila/encoda/dist/codecs/pandoc/binary stencila-deps
$CP node_modules/@stencila/encoda/dist/codecs/pandoc/templates stencila-deps
$CP node_modules/@stencila/encoda/dist/codecs/tex/*.xsl stencila-deps
$CP node_modules/puppeteer/.local-chromium stencila-deps
$CP node_modules/opn/xdg-open stencila-deps

tar czf stencila-deps.tgz stencila-deps
rm -r stencila-deps
