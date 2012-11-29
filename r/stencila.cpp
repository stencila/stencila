#include <Rcpp.h>

//Use Rcpp as (conversion from R to C++) and wrap (conversion from C++ to R)
//functions for converting standard types, eg strings, vectors etc
using Rcpp::as;
using Rcpp::wrap;

//For Alpha classes use the usual "externalptr" mechanism
//but with the addition of externalptr's "tag" member. this allows
//type information to be stored on the R side for appropriate conversion to an R class

//Conversion of Stencila objects between C++ and R
//Uses Rcpp::XPtr
//An alternative here would be to use the R API functions
//	R_MakeExternalPtr
//  R_SetExternalPtrTag
//  R_SetExternalPtrProtected
//	R_RegisterCFinalizerEx
//See the following
//	http://dirk.eddelbuettel.com/code/rcpp/html/XPtr_8h_source.html
//	http://stackoverflow.com/questions/7032617/storing-c-objects-in-r
//However, there appears to be no benefit of dealing with complexity of that,
//so just use Rcpp's constructor

//Conversion from C++ to R
template<typename Type>
SEXP to(Type* object, const char* clazz){
	return Rcpp::XPtr<Type>(object,true,wrap(clazz));
}
//Conversion from R to C++
template<typename Type>
Type& from(SEXP self){
    // An alternative is to use 
    //  return *static_cast<Type*>(R_ExternalPtrAddr(self))
    // however that does not protect the pointer from R garbage collection
    // using Rcpp::XPtr does. Note though that we need to provide the exisiting tag
    // because otherwise the XPtr constructor sets it with a default R_NilValue
    return *Rcpp::XPtr<Type>(self,R_ExternalPtrTag(self));
}

///////////////////////////////

const decltype(R_NilValue) nil = R_NilValue;

//Some macros for consistency.

#define STENCILA_R_TO(TYPE,POINTER) \
	to<TYPE>(POINTER,#TYPE)

#define STENCILA_R_FUNC extern "C" SEXP

//The following are based on BEGIN_RCPP and END_RCPP macros
#define STENCILA_R_TRY \
    try {
        
#define STENCILA_R_CATCH \
    } catch(std::exception& __ex__ ){ \
        forward_exception_to_r( __ex__ ); \
    } catch(...){ \
        ::Rf_error( "Unknown C++ exception" ); \
    }

#define STENCILA_R_BEGIN \
    try {

#define STENCILA_R_END \
    STENCILA_R_CATCH \
    return nil;

////////////////////////////////////////////////////

#include <stencila/version.hpp>
STENCILA_R_FUNC Stencila_version(void){
    //! Return the version of the Stencila library
	STENCILA_R_BEGIN
		return wrap(Stencila::version);
	STENCILA_R_END
}

STENCILA_R_FUNC Stencila_class(SEXP self){
	//! Obtain the class name from the tag of an "externalpointer"
    STENCILA_R_BEGIN
        return R_ExternalPtrTag(self);
    STENCILA_R_END
}

#include "datacursor.hpp"
#include "dataquery.hpp"
#include "datatable.hpp"
#include "dataset.hpp"
#include "stencil.hpp"

#include <stencila/dataset.cpp>


