#include <stencila/stencil.hpp>
using namespace Stencila;

#include "stencila.hpp"
#include "context.hpp"

STENCILA_R_FUNC Stencil_new(void){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Stencil,new Stencil);
    STENCILA_R_END
}

STENCILA_R_FUNC Stencil_content_get(SEXP self, SEXP language){
    STENCILA_R_BEGIN
        return wrap(from<Stencil>(self).content(
            as<std::string>(language)
        ));
    STENCILA_R_END
}

STENCILA_R_FUNC Stencil_content_set(SEXP self, SEXP content, SEXP language){
    STENCILA_R_BEGIN
        from<Stencil>(self).content(
            as<std::string>(content),
            as<std::string>(language)
        );
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Stencil_render(SEXP self,SEXP context){
    STENCILA_R_BEGIN
        RContext rcontext(context);
        from<Stencil>(self).render(rcontext);
        return nil;
    STENCILA_R_END
}
