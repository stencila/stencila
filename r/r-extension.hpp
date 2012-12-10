#pragma once

#include <Rcpp.h>

//Use Rcpp as (conversion from R to C++) and wrap (conversion from C++ to R)
//functions for converting standard types, eg strings, vectors etc
using Rcpp::as;
using Rcpp::wrap;

//! @{
//! @name Macros etc
//! Some macros and constants used for brevity and consistency in coding R functions.
//! STENCILA_R_TRY and STENCILA_R_CATCH based on BEGIN_RCPP and END_RCPP macros.

const decltype(R_NilValue) nil = R_NilValue;

#define STENCILA_R_TO(TYPE,POINTER) \
	to<TYPE>(POINTER,#TYPE)

#define STENCILA_R_FUNC extern "C" SEXP

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
    
//! @}

/*! 
@{
@name Conversion functions

Conversion of Stencila objects between C++ and R

For Stencila classes use the usual "externalptr" mechanism
but with the addition of externalptr's "tag" member. this allows
type information to be stored on the R side for appropriate conversion to an R class

Uses Rcpp::XPtr. An alternative here would be to use the R API functions
    R_MakeExternalPtr
    R_SetExternalPtrTag
    R_SetExternalPtrProtected
    R_RegisterCFinalizerEx
See the following
	http://dirk.eddelbuettel.com/code/rcpp/html/XPtr_8h_source.html
	http://stackoverflow.com/questions/7032617/storing-c-objects-in-r
However, there appears to be no benefit of dealing with complexity of that,
so just use Rcpp's constructor
*/

//! Conversion from C++ to R
template<typename Type>
SEXP to(Type* object, const char* clazz){
	return Rcpp::XPtr<Type>(object,true,wrap(clazz));
}

//! Conversion from R to C++
template<typename Type>
Type& from(SEXP self){
    // An alternative is to use 
    //  return *static_cast<Type*>(R_ExternalPtrAddr(self))
    // however that does not protect the pointer from R garbage collection
    // using Rcpp::XPtr does. Note though that we need to provide the exisiting tag
    // because otherwise the XPtr constructor sets it with a default R_NilValue
    return *Rcpp::XPtr<Type>(self,R_ExternalPtrTag(self));
}

//! @}
