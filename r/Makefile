all: regen lint test build docs

setup:
	Rscript -e "install.packages('devtools'); devtools::install_dev_deps()"

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
	Rscript -e 'devtools::document(); pkgdown::build_site()'
.PHONY: docs

install: build
	Rscript -e 'devtools::install()'

clean:
	rm -rf man stencilaschema_*.tar.gz stencilaschema.Rcheck
