all: format lint build

setup:
	Rscript -e "install.packages('remotes', repos = 'https://cloud.r-project.org')"
	Rscript -e "remotes::install_deps(dependencies = TRUE, force = TRUE)"

format:
	cargo fmt

lint:
	touch src/*.rs
	cargo clippy

check:
	Rscript -e 'devtools::check()'

audit:
	cargo +nightly udeps

build:
	Rscript -e 'devtools::document()'
	Rscript -e 'devtools::build()'

install: build
	R CMD INSTALL .
