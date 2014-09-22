#include <stencila/component.hpp>

#include "extension.hpp"
#include "r-context.hpp"

// Include R internals for turning off
// R's C stack limit checking below
#define CSTACK_DEFNS 7
#include "Rinterface.h"

using namespace Stencila;

/**
 * Start up function for the Stencila R module
 */
STENCILA_R_FUNC Stencila_startup(void){

	// Turn off R's C stack limit checking so does not crash the sesssion
	// when rendering via stencils via the server. This appears to happen
	// because the server thread is attempting to access the R-side context.
	// e.g. http://stats.blogoverflow.com/2011/08/using-openmp-ized-c-code-with-r/
	R_CStackLimit = (uintptr_t)-1;

	Component::classes();

	Component::class_(Component::RContextType, Component::Class(
		"RContext",
		RContext::page,
		RContext::call
	));

	return null;
}

/**
 * Shutdown function for the Stencila R module
 */
STENCILA_R_FUNC Stencila_shutdown(void){
	return null;
}

/**
 * Get the Stencila class name from the tag of an "externalpointer" in R
 *
 * This is used when converting an externalpointer returned from a call to a
 * C++ function into an R-side class
 * 
 * @param  self The object to obtain the tag for
 */
STENCILA_R_FUNC Stencila_class(SEXP self){
    STENCILA_R_BEGIN
        return R_ExternalPtrTag(self);
    STENCILA_R_END
}
