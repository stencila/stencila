#include <stencila/stencil.hpp>

using namespace Stencila;

#include "extension.hpp"
#include "r-context.hpp"

STENCILA_R_FUNC Context_new(SEXP context){
    STENCILA_R_BEGIN
        return to<RContext>(new RContext(context),"Context");
    STENCILA_R_END
}

// Overrides of Component methods

STENCILA_R_FUNC Context_serve(SEXP self){
    STENCILA_R_BEGIN
        return wrap(from<RContext>(self).serve());
    STENCILA_R_END
}

STENCILA_R_FUNC Context_view(SEXP self){
    STENCILA_R_BEGIN
        from<RContext>(self).view();
        return null;
    STENCILA_R_END
}
