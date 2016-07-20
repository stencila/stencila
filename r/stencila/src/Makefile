# A makefile for Linux and Mac OSX

# Compiler
CXX=g++
CXX_FLAGS := $(shell R CMD config --cppflags) $(shell Rscript -e "Rcpp:::CxxFlags()") \
	-std=c++11 \
	-I../../../cpp \
	-I../../../cpp/build/requires/boost \
	-I../../../cpp/build/requires/websocketpp

# Linker
LD=g++
LD_FLAGS := $(shell R CMD config --ldflags) $(shell Rscript -e "Rcpp:::LdFlags()")
LD_LIBS := -L../../../cpp/build -lstencila -lcurl -lssl -lcrypto -lws2_32 -lmswsock -lssh2 -lz

# List of compiled object files
OBJECTS = $(patsubst %.cpp,%.o,$(wildcard *.cpp))

all: stencila.so $(DLLS)

%.o: %.cpp
	$(CXX) $(CXX_FLAGS) -c -o$@ $<

stencila.so: $(OBJECTS)
	$(LD) $(LD_FLAGS) -shared -o$@ $^ $(LD_LIBS)
