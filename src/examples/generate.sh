#!/bin/sh

#npx encoda convert https://elifesciences.org/articles/49574v2 article-drosophila.html

#npx encoda convert 'https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1007207' article-antibodies.html

curl -o rmarkdown.nb.html https://raw.githubusercontent.com/stencila/examples/master/rmarkdown/rmarkdown.nb.html
npx encoda convert  rmarkdown.nb.html --from rnb article-rmarkdown.html
rm rmarkdown.nb.html
