#include <stencila/stencil.hpp>
using namespace Stencila;

#include "extension.hpp"
#include "r-context.hpp"

STENCILA_R_NEW(Stencil)

// Overrides of Component methods

STENCILA_R_RET0(Stencil,serve) 
STENCILA_R_EXEC0(Stencil,view)

// Content getters and setters

STENCILA_R_FUNC Stencil_content_get(SEXP self, SEXP format){
    STENCILA_R_BEGIN
        return wrap(from<Stencil>(self).content(
            as<std::string>(format)
        ));
    STENCILA_R_END
}
STENCILA_R_FUNC Stencil_content_set(SEXP self, SEXP format, SEXP content){
    STENCILA_R_BEGIN
        from<Stencil>(self).content(
            as<std::string>(format),
            as<std::string>(content)
        );
        return null;
    STENCILA_R_END
}

STENCILA_R_ATTR(Stencil,html,std::string)

// Contexts and rendering

STENCILA_R_ATTR(Stencil,contexts,std::vector<std::string>)

STENCILA_R_FUNC Stencil_render(SEXP self,SEXP context){
    STENCILA_R_BEGIN
        RContext rcontext(context);
        from<Stencil>(self).render(rcontext);
        return null;
    STENCILA_R_END
}
