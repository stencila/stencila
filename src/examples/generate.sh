#!/bin/sh

# Generate article-kitchen-sink.html
npx encoda convert article-kitchen-sink.json article-kitchen-sink.html --standalone=false

# Generate article-drosophila.html
npx encoda convert https://elifesciences.org/articles/49574v2 article-drosophila.html --standalone=false

# Generate article-antibodies.html
npx encoda convert 'https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1007207' article-antibodies.html --standalone=false

# Generate article-rmarkdown.html
# Temporarily excluded becuase, with removal of `coerce()` call from `xmd` codec
# the metadata is causing this to fail.
# curl -o rmarkdown.nb.html https://raw.githubusercontent.com/stencila/examples/master/rmarkdown/rmarkdown.nb.html
# npx encoda convert  rmarkdown.nb.html --from rnb article-rmarkdown.html --standalone=false
# rm rmarkdown.nb.html
