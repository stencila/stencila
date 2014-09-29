// Include R internals for turning off
// R's C stack limit checking below
// Included here because including "Rinterface.h" along with `cpp/stencila/component.hpp'
// causes the error 
//   "/usr/share/R/include/Rinterface.h:89:24: error: conflicting declaration ‘typedef long unsigned int uintptr_t’"
#define CSTACK_DEFNS 7
#include "Rinterface.h"

void Stencila_R_CStackLimit(void){
	// Turn off R's C stack limit checking so does not crash the sesssion
	// when rendering via stencils via the server. This appears to happen
	// because the server thread is attempting to access the R-side context.
	// e.g. http://stats.blogoverflow.com/2011/08/using-openmp-ized-c-code-with-r/
	// 
	R_CStackLimit = (uintptr_t)-1;
}