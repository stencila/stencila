include ../cpp/Makefile.inc

.PHONY: compile buiild check

all:  clean compile build check

#Requires:
#	sudo R
#	install.packages('Rcpp','/usr/local/lib/R/site-library')
#/usr/local/lib/R/site-library is used so that other uses (e.g. build bots can use Rcpp)
#For some of these compiler flags see
#Rcpp:::LdFlags()
RCPP_HOME := /usr/lib/R/site-library/Rcpp
compile:
	cd stencila/src; \
	g++ -I/usr/share/R/include  -I$(RCPP_HOME)/include $(STENCILA_CPP_FLAGS) -fpic  -O3 -pipe  -g -c stencila.cpp -o stencila.o ; \
	g++ -shared -o stencila.so stencila.o \
		-L /usr/lib/R/lib -lR \
		-L$(RCPP_HOME)//lib -lRcpp -Wl,-rpath,$(RCPP_HOME)/lib \
		$(STENCILA_CPP_LIBS)
		
build:
	R CMD build stencila
	
check:
	R CMD check stencila

clean:
	rm -rf stencila_*.tar.gz stencila.Rcheck