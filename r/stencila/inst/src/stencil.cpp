#include <stencila/stencil.hpp>
using namespace Stencila;

#include "stencila.hpp"
#include "context.hpp"

STENCILA_R_NEW(Stencil)

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

STENCILA_R_ATTR(Stencil,contexts,std::vector<std::string>)

STENCILA_R_FUNC Stencil_render(SEXP self,SEXP context){
    STENCILA_R_BEGIN
        RContext rcontext(context);
        from<Stencil>(self).render(rcontext);
        return null;
    STENCILA_R_END
}
