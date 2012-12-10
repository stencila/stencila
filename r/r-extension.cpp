#include <stencila/version.hpp>
using namespace Stencila;

#include "r-extension.hpp"

//! Get the version number of the Stencila library
STENCILA_R_FUNC Stencila_version(void){
	STENCILA_R_BEGIN
		return wrap(Stencila::version);
	STENCILA_R_END
}

//! Get the Stencila class name from the tag of an "externalpointer" in R
STENCILA_R_FUNC Stencila_class(SEXP self){
    STENCILA_R_BEGIN
        return R_ExternalPtrTag(self);
    STENCILA_R_END
}