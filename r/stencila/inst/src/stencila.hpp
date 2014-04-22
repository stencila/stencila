#pragma once

#include <Rcpp.h>

/*
Conversion of atomic and standard library objects eg strings, vectors etc
between C++ and R.

Use Rcpp as (conversion from R to C++) and wrap (conversion from C++ to R).
*/
using Rcpp::as;
using Rcpp::wrap;

/*
Conversion of Stencila objects between C++ and R

For Stencila classes use the usual "externalptr" mechanism
but with the addition of externalptr's "tag" member. This allows
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
#define STENCILA_R_USE_RCPP_XPTR 1

#if !STENCILA_R_USE_RCPP_XPTR
    // R finaliser for R_RegisterCFinalizerEx
    template<typename Type>
    static void finalizer(SEXP pointer){
        delete static_cast<Type*>(R_ExternalPtrAddr(pointer));
        R_ClearExternalPtr(pointer);
    }
#endif

// Conversion from C++ to R
template<typename Type>
SEXP to(Type* object, const char* classs){
    #if STENCILA_R_USE_RCPP_XPTR
        // The signature for this XPtr constructor is:
        //    Rcpp::XPtr(T* p, bool set_delete_finalizer = true, SEXP tag = R_NilValue, SEXP prot = R_NilValue)
        return Rcpp::XPtr<Type>(object,true,wrap(classs));
    #else
        SEXP pointer = PROTECT(R_MakeExternalPtr(object, wrap(classs), R_NilValue));
        R_RegisterCFinalizerEx(pointer, finalizer<Type>, TRUE);
        UNPROTECT(1);
        return pointer;
    #endif
}

// Conversion from R to C++
template<typename Type>
Type& from(SEXP self){
    #if STENCILA_R_USE_RCPP_XPTR
        // The signature for this XPtr constructor is:
        //    Rcpp::XPtr(SEXP m_sexp, SEXP tag = R_NilValue, SEXP prot = R_NilValue)
        // Note though that we need to provide the exisiting tag
        // because otherwise the XPtr constructor sets it with a default R_NilValue
        return *Rcpp::XPtr<Type>(self,R_ExternalPtrTag(self));
    #else
        return *static_cast<Type*>(R_ExternalPtrAddr(self));
    #endif
}

/*
Macros and constants used for brevity and consistency in coding R functions.
*/

const decltype(R_NilValue) null = R_NilValue;

// Convert from C++ to R
#define STENCILA_R_TO(TYPE,POINTER) \
	to<TYPE>(POINTER,#TYPE)

#define STENCILA_R_FUNC extern "C" SEXP

// Define try/catch blocks
// STENCILA_R_TRY and STENCILA_R_CATCH based on BEGIN_RCPP and END_RCPP macros.
#define STENCILA_R_TRY \
    try {
        
#define STENCILA_R_CATCH \
    } catch(std::exception& __ex__ ){ \
        forward_exception_to_r( __ex__ ); \
    } catch(...){ \
        ::Rf_error( "Unknown C++ exception" ); \
    }

// Define begin and end of a funciton
#define STENCILA_R_BEGIN \
    try {

#define STENCILA_R_END \
    STENCILA_R_CATCH \
    return null;

// Define a `new` method for a class
#define STENCILA_R_NEW(CLASS) \
STENCILA_R_FUNC CLASS##_new(void){ \
    STENCILA_R_BEGIN \
        return STENCILA_R_TO(CLASS,new CLASS); \
    STENCILA_R_END \
}

// Call a method with 0 arguments and return value
#define STENCILA_R_RET0(CLASS,NAME) \
STENCILA_R_FUNC CLASS##_##NAME(SEXP self){ \
    STENCILA_R_BEGIN \
        return wrap(from<CLASS>(self).NAME()); \
    STENCILA_R_END \
}

// Call a method with 0 arguments and return `null`
// (`null` is converted to `self` on R side so method chaining can
// be used)
#define STENCILA_R_EXEC0(CLASS,NAME) \
STENCILA_R_FUNC CLASS##_##NAME(SEXP self){ \
    STENCILA_R_BEGIN \
        from<CLASS>(self).NAME(); \
        return null; \
    STENCILA_R_END \
}

// Call a method with 1 arguments and return `null`
#define STENCILA_R_EXEC1(CLASS,NAME,TYPE) \
STENCILA_R_FUNC CLASS##_##NAME(SEXP self, SEXP arg1){ \
    STENCILA_R_BEGIN \
        from<CLASS>(self).NAME(as<TYPE>(arg1)); \
        return null; \
    STENCILA_R_END \
}

// Define a `_get` method for an 
#define STENCILA_R_GET(CLASS,NAME) \
STENCILA_R_FUNC CLASS##_##NAME##_get(SEXP self){ \
    STENCILA_R_BEGIN \
        return wrap(from<CLASS>(self).NAME()); \
    STENCILA_R_END \
}

// Define a `_set` method for an attribute
#define STENCILA_R_SET(CLASS,NAME,TYPE) \
STENCILA_R_FUNC CLASS##_##NAME##_set(SEXP self, SEXP arg1){ \
    STENCILA_R_BEGIN \
        from<CLASS>(self).NAME(as<TYPE>(arg1)); \
        return null; \
    STENCILA_R_END \
}

// Define both get an set for an attribute
#define STENCILA_R_ATTR(CLASS,NAME,TYPE) \
    STENCILA_R_GET(CLASS,NAME)\
    STENCILA_R_SET(CLASS,NAME,TYPE)
