#include <stencila/version.hpp>
using namespace Stencila;

#include "stencila.hpp"

/**
 * Start up function for the Stencila R module
 */
STENCILA_R_FUNC Stencila_startup(void){
	return nil;
}

/**
 * Shutdown function for the Stencila R module
 */
STENCILA_R_FUNC Stencila_shutdown(void){
	return nil;
}

/**
 * Get the version number of the Stencila library
 */
STENCILA_R_FUNC Stencila_version(void){
    STENCILA_R_BEGIN
        return wrap(Stencila::version);
    STENCILA_R_END
}
