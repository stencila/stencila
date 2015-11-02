#!/bin/sh

echo "Creating build directories"
mkdir -p build/fonts


echo "Setting font-awesome 'fa-font-path' variable"
sed -i '' 's!\$fa-font-path: .*$!\$fa-font-path: "/web/fonts/";!' node_modules/font-awesome/scss/_variables.scss

echo "Copying font-awesome fonts"
cp -f node_modules/font-awesome/fonts/* build/fonts


echo "Building Ace editor"
cd node_modules/ace && npm install
