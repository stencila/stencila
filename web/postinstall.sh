#!/bin/sh


echo "Setting font-awesome 'fa-font-path' variable"
sed -i.back 's!\$fa-font-path: .*$!\$fa-font-path: "/web/fonts/";!' node_modules/font-awesome/scss/_variables.scss

echo "Copying font-awesome fonts"
mkdir -p build/fonts
cp -f node_modules/font-awesome/fonts/* build/fonts


echo "Installing Ace editor"
cd node_modules/ace
npm install
cd ../..

if [ ! -e node_modules/ace/lib/ace/mode/cila.js ]; then
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
fi

if [ ! -e node_modules/ace/build/src-min ]; then
	echo "Building Ace editor"
	cd node_modules/ace
	node Makefile.dryice.js -m
	cd ../..
fi

if [ ! -e build/ace/src-min ]; then
	echo "Copying Ace editor build"
	rm -rf build/ace
	mkdir -p build/ace
	cp -rf node_modules/ace/build/src-min build/ace
fi
