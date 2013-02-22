#include <stencila/version.hpp>
using namespace Stencila;

#include "r-context.hpp"
#include "r-extension.hpp"

//! @brief Start up function for the Stencila R module
STENCILA_R_FUNC Stencila_startup(void){
    //Declare component types
    Component<>::declarations();
    Component<>::declare<RContext>();
    return nil;
}

//! @brief Get the version number of the Stencila library
STENCILA_R_FUNC Stencila_version(void){
    STENCILA_R_BEGIN
        return wrap(Stencila::version);
    STENCILA_R_END
}

//! @brief Get the Stencila class name from the tag of an "externalpointer" in R
STENCILA_R_FUNC Stencila_class(SEXP self){
    STENCILA_R_BEGIN
        return R_ExternalPtrTag(self);
    STENCILA_R_END
}