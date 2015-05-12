#include <stencila/component.hpp>
#include <stencila/network.hpp>
#include <stencila/host.hpp>

#include "extension.hpp"
#include "context.hpp"

using namespace Stencila;

void Stencila_R_CStackLimit(void);

/**
 * Start up function for the Stencila R module
 */
STENCILA_R_FUNC Stencila_startup(void){

	Stencila_R_CStackLimit();

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
	// Shutdown server if it has been started
	Server::shutdown();
	return null;
}

/**
 * Get the Stencila home directory
 */
STENCILA_R_FUNC Stencila_home(void){
	return wrap(Host::home_dir());
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
