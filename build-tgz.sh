#!/bin/bash

set -e

ARCHIVE_NAME=stencila-deps.tgz

mkdir -p archive-source/node_modules/puppeteer/

cp -R node_modules/@stencila/encoda/src archive-source/ 
cp -R node_modules/@stencila/encoda/vendor archive-source/ 
cp -R node_modules/puppeteer/.local-chromium archive-source/node_modules/puppeteer/

cd archive-source
tar czf ${ARCHIVE_NAME} src *
cd ..
mv archive-source/${ARCHIVE_NAME} .
rm -r archive-source