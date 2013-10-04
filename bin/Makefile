all: stencila

include ../cpp/cpp.mk
	
stencila: stencila.cpp
	$(CXX) $(STENCILA_CPP_FLAGS) -O2 $(STENCILA_CPP_INCLUDE_DIRS) -ostencila stencila.cpp $(STENCILA_CPP_LIB_DIRS) $(STENCILA_CPP_LIBS) -lboost_program_options

clean:
	rm -f stencila
