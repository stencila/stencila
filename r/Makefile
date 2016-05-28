include ../shared.make


build:
	sed -i 's!Version: .*$$!Version: $(shell ../version.sh r)!' DESCRIPTION
	sed -i 's!Date: .*$$!Date: $(shell date -u +%Y-%m-%dT%H:%M:%SZ)!' DESCRIPTION
	Rscript -e "library(roxygen2); roxygenize('.');"
	R CMD check .
ifeq ($(OS),win)
	R CMD INSTALL --build .
else
	R CMD build .
endif

# Clean files created during build
clean:
	rm -rf NAMESPACE man *.Rcheck *.tar.gz
