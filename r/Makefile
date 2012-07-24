include ../cpp/Makefile.inc

.PHONY: compile buiild check test

all:  compile build check test

#Requires:
#	sudo R
#	install.packages('Rcpp','/usr/local/lib/R/site-library')
#/usr/local/lib/R/site-library is used so that other uses (e.g. build bots can use Rcpp)
#For some of these compiler flags see
#Rcpp:::LdFlags()
R_RCPP_HOME := /usr/local/lib/R/site-library/Rcpp
compile:
	cd stencila/src; \
	g++ -I/usr/share/R/include  -I$(R_RCPP_HOME)/include -std=c++0x -fpic  -O3 -pipe  -g -c stencila.cpp -o stencila.o ; \
	g++ -shared -o stencila.so stencila.o \
		-L /usr/lib/R/lib -lR \
		-L$(R_RCPP_HOME)//lib -lRcpp -Wl,-rpath,$(R_RCPP_HOME)/lib \
		$(CPP_LIBS)
		
build:
	R CMD build stencila
	
check:
	R CMD check stencila

test:
	cd tests;\
	R --vanilla < tests.R

clean:
	rm -f stencila_*.tar.gz stencila.Rcheck