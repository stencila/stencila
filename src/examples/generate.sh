#!/bin/sh

# Generate article-kitchen-sink.html
npx encoda convert article-kitchen-sink.json article-kitchen-sink.html

# Generate article-drosophila.html
npx encoda convert https://elifesciences.org/articles/49574v2 article-drosophila.html

# Generate article-antibodies.html
npx encoda convert 'https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1007207' article-antibodies.html

# Generate article-rmarkdown.html
# Temporarily excluded becuase, with removal of `coerce()` call from `xmd` codec
# the metadata is causing this to fail.
# curl -o rmarkdown.nb.html https://raw.githubusercontent.com/stencila/examples/master/rmarkdown/rmarkdown.nb.html
# npx encoda convert  rmarkdown.nb.html --from rnb article-rmarkdown.html
# rm rmarkdown.nb.html

# Replace unpkg Thema packages with local versions for development. Swapping out JavaScript files for TypeScript
perl -pi -w -e 's/https:\/\/unpkg.com\/\@stencila\/thema\@1\/dist\/(.+\/index)\.js/\.\.\/$1\.ts/g;' *.html
perl -pi -w -e 's/https:\/\/unpkg.com\/\@stencila\/thema\@1\/dist/\.\./g;' *.html
