all: setup clean lint test build docs

setup:
	Rscript -e "install.packages('devtools')"
	Rscript -e "devtools::install_github(c('jimhester/lintr', 'klutometis/roxygen', 'r-lib/covr', 'r-lib/testthat'))"

lint:
	Rscript -e 'lintr::lint_package()'

test:
	Rscript -e 'devtools::test()'

cover:
	Rscript -e 'devtools::document()'
	Rscript -e 'covr::package_coverage()'

build:
	R CMD build . && R CMD check *.tar.gz

docs:
	Rscript -e 'devtools::document()'

clean:
	rm -rf man stencilaschema_*.tar.gz stencilaschema.Rcheck
