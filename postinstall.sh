#!/bin/sh

# A "postinstall" script used by npm to copy stuff from `node_modules`
# to `build` (where they are expected to be by development server and 
# with relative paths as used in production)
 
mkdir -p build

# Fonts
echo "Copying fonts in repo"
mkdir -p build/fonts
cp -f fonts/* build/fonts

# font-awesome
echo "Setting font-awesome 'fa-font-path' variable"
sed -i.back 's!\$fa-font-path: .*$!\$fa-font-path: "fonts";!' node_modules/font-awesome/scss/_variables.scss

echo "Copying font-awesome fonts"
mkdir -p build/fonts
cp -f node_modules/font-awesome/fonts/* build/fonts


# Ace editor
if [ ! -e build/ace ]; then
	echo "Installing Ace editor"
	cd node_modules/ace
		npm install
	cd ../..

	echo "Building Ace editor"
	cd node_modules/ace
		node Makefile.dryice.js -m
	cd ../..

	echo "Copying Ace editor build"
	rm -rf build/ace
	cp -rf node_modules/ace/build/src-min build/ace

fi

# KaTeX
# Currently CSS files are not bundled into the npm install for katex
# See https://github.com/Khan/KaTeX/issues/148
# So download build with compiled CSS and fonts into build/katex
if [ ! -e build/katex ]; then
	echo "Getting KaTeX CSS & fonts"
	curl -o build/katex.tar.gz -L https://github.com/Khan/KaTeX/releases/download/v0.6.0/katex.tar.gz
	tar xzf build/katex.tar.gz -C build
	rm build/katex.tar.gz
fi

# EmojiOne
# Allow PNGs to be served locally
echo "Copying EmojiOne PNGs"
rm -rf build/emojione
mkdir -p build/emojione
cp -rf node_modules/emojione/assets/png build/emojione/png


 

