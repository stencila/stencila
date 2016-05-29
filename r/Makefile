include ../shared.make


build:
	sed -i 's!Version: .*$$!Version: $(shell ../version.sh r)!' stencila/DESCRIPTION
	sed -i 's!Date: .*$$!Date: $(shell date -u +%Y-%m-%dT%H:%M:%SZ)!' stencila/DESCRIPTION
	Rscript -e "library(roxygen2); roxygenize('stencila');"
	R CMD check stencila
ifeq ($(OS),win)
	R CMD INSTALL --build stencila
else
	R CMD build stencila
endif

install:
	R CMD INSTALL stencila
	sudo Rscript -e 'library(stencila); stencila:::install()'

clean:
	rm -rf NAMESPACE man *.Rcheck *.tar.gz
