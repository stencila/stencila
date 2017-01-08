#!/bin/bash

# A build script to build and copy stuff to `build` 
# Eventually this will probably get integrated innto `gulpfile.js`
 
mkdir -p build

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

# Sematic UI
if [ ! -e build/themes/default ]; then
  echo "Copying Semantic UI default theme"
  cp -rf node_modules/semantic-ui-css/themes build
fi

# Test examples
echo "Building test examples"
mkdir -p build/tests/document/nodes
for example in 'nodes/all' 'nodes/blockquote' 'nodes/codeblock' 'nodes/default' \
               'nodes/discussion' 'nodes/emoji' 'nodes/emphasis'  'nodes/heading' \
               'nodes/image' 'nodes/link' 'nodes/math' 'nodes/paragraph' 'nodes/print' \
               'nodes/strong' 'nodes/summary' 'nodes/title'
do
  cat <(echo '<!DOCTYPE html>
<html>
  <head>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="stylesheet" type="text/css" href="../../../document.min.css">
  </head>
  <body>
    <main id="data" data-format="html">
      <div class="content">') tests/document/$example/index.html <(echo '
  	  </div>
    <script>window.stencila = {root: "../../.."}</script>
    <script src="../../../document.min.js"></script>
    </main>
  </body>
</html>') > build/tests/document/$example.html
done
