#!/bin/sh


echo "Setting font-awesome 'fa-font-path' variable"
sed -i.back 's!\$fa-font-path: .*$!\$fa-font-path: "/web/fonts/";!' node_modules/font-awesome/scss/_variables.scss

echo "Copying font-awesome fonts"
mkdir -p build/fonts
cp -f node_modules/font-awesome/fonts/* build/fonts

if [ ! -e node_modules/ace-built.flag ]; then
	echo "Installing Ace editor"
	cd node_modules/ace
		npm install
	cd ../..

	echo "Patching Ace editor with Cila mode and themes"
	ln -sf "$(pwd)/ace/mode/cila.js" node_modules/ace/lib/ace/mode/cila.js
	ln -sf "$(pwd)/ace/mode/cila_highlight_rules.js" node_modules/ace/lib/ace/mode/cila_highlight_rules.js
	ln -sf "$(pwd)/ace/theme/cilacon.js" node_modules/ace/lib/ace/theme/cilacon.js
	ln -sf "$(pwd)/ace/theme/cilacon.css" node_modules/ace/lib/ace/theme/cilacon.css
	ln -sf "$(pwd)/ace/snippets/cila.js" node_modules/ace/lib/ace/snippets/cila.js
	ln -sf "$(pwd)/ace/snippets/cila.snippets" node_modules/ace/lib/ace/snippets/cila.snippets
	ln -sf "$(pwd)/ace/cila.cila" node_modules/ace/demo/kitchen-sink/docs/cila.cila
	sed -i.back 's/Cirru/Cila:\["cila"\],\n    Cirru/' node_modules/ace/lib/ace/ext/modelist.js
	sed -i.back 's/\["Clouds Midnight"/\["Cilacon","cilacon","dark"\],\n    ["Clouds Midnight"/' node_modules/ace/lib/ace/ext/themelist.js
	sed -i.back 's/dark: \[/dark: \["cilacon",/' node_modules/ace/tool/mode_creator.js

	echo "Building Ace editor"
	cd node_modules/ace
		node Makefile.dryice.js -m
	cd ../..

	echo "Copying Ace editor build"
	cp -rf node_modules/ace/build/src-min build/ace

	touch node_modules/ace-built.flag
fi

if [ ! -e node_modules/mathjax ]; then
	echo "Getting MathJax"
	# It would be nice to just put this as a node dependency and let it download it
	# but it did not like it for some strange reason
	MATHJAX_VERSION=2.5.3
	wget --no-check-certificate -O node_modules/mathjax.tar.gz https://github.com/mathjax/MathJax/archive/$MATHJAX_VERSION.tar.gz
	tar xf node_modules/mathjax.tar.gz -C node_modules
	mv node_modules/MathJax-$MATHJAX_VERSION node_modules/mathjax
fi

echo "Shrinking MathJax"
#  See https://github.com/mathjax/MathJax-docs/wiki/Guide%3A-reducing-size-of-a-mathjax-installation
cd node_modules/mathjax
	rm -rf docs localization test unpacked .gitignore README-branch.txt README.md bower.json
	rm -rf `find config -name '*.js' ! -name 'TeX-MML-AM_HTMLorMML.js'`
	rm -rf fonts/HTML-CSS
	rm -rf jax/output/SVG
	rm -rf `find jax/output/HTML-CSS/fonts -mindepth 1 -maxdepth 1 ! -name 'STIX'`
cd ../..

echo "Copying MathJax build"
cp -rf node_modules/mathjax build/mathjax
