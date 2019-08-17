all: setup clean lint test build docs

setup:
	Rscript -e "install.packages('devtools')"
	Rscript -e "devtools::install_github(c('jimhester/lintr', 'klutometis/roxygen', 'r-lib/covr', 'r-lib/testthat'))"

regen:
	npm run build:r

lint:
	Rscript -e 'lintr::lint_package()'

test:
	Rscript -e 'devtools::test()'

autotest:
	Rscript -e 'testthat::auto_test_package()'

cover:
	Rscript -e 'covr::package_coverage()'

check:
	Rscript -e 'devtools::check()'

build:
	Rscript -e 'devtools::document(); warnings()'
	Rscript -e 'devtools::build()'

docs:
	Rscript -e 'devtools::document()'

clean:
	rm -rf man stencilaschema_*.tar.gz stencilaschema.Rcheck
