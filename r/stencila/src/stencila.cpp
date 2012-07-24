#include <Rcpp.h>

//Use Rcpp as (conversion from R to C++) and wrap (conversion from C++ to R)
//functions for converting standard types, eg strings, vectors etc
using Rcpp::as;
using Rcpp::wrap;

//For Alpha classes use the usual "externalptr" mechanism
//but with the addition of externalptr's "tag" member. this allows
//type information to be stored on the R side for appropriate conversion to an R class

//Conversion from C++ to R
template<typename Type>
SEXP to(Type* object, std::string type){
	//An alternative here would be to use the R API functions
	//	R_MakeExternalPtr
	//	R_RegisterCFinalizerEx
	//See the following
	//	http://dirk.eddelbuettel.com/code/rcpp/html/XPtr_8h_source.html
	//	http://stackoverflow.com/questions/7032617/storing-c-objects-in-r
	//Appears to be no benefit of dealing with complexity of that though,
	//so just use Rcpp's constructor
	return Rcpp::XPtr<Type>(object,true,wrap(type));
}

//Conversion from R to C++
template<typename Type>
Type& from(SEXP self){
	//An alternative here would be to use 
	//	Rcpp::XPtr<Alpha::Dataset>(self)
	//and get the tag from it.
	//But when I tried that it seemed to cause the loss of tag information.
	//So, just use the standard R API function
	return *static_cast<Type*>(R_ExternalPtrAddr(self));
}

//Function exported to R for obtaining tag from an "externalpointer"
extern "C" SEXP tag(SEXP self){
	return R_ExternalPtrTag(self);
}

//Some macros for brevity.
//Defined after all includes to prevent conflicts
#define TO(TYPE,POINTER) \
	to<TYPE>(POINTER,#TYPE)
#define NIL R_NilValue;
#define EXPORT extern "C"
#define BEGIN BEGIN_RCPP
#define END END_RCPP

#include "datacursor.cpp"
#include "datatable.cpp"
#include "dataset.cpp"
