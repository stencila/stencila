include ../cpp/Makefile.inc

.PHONY: compile build check

all:  build check

# The compile job is for testing compilation without doing the full "R CMD check"
RCPP_CXXFLAGS := $(shell Rscript -e "Rcpp:::CxxFlags()")
RCPP_LDFLAGS :=  $(shell Rscript -e "Rcpp:::LdFlags()")
compile:
	cd stencila/src;\
	g++ -I/usr/share/R/include $(RCPP_CXXFLAGS) $(STENCILA_CPP_FLAGS) -fpic -g -c stencila.cpp -o stencila.o;\
	g++ -shared -o stencila.so stencila.o -L/usr/lib/R/lib -lR $(RCPP_LDFLAGS) $(STENCILA_CPP_LIBS);\
		
build:
	R CMD build stencila
	
check:
	R CMD check stencila

clean:
	rm -rf stencila_*.tar.gz stencila.Rcheck