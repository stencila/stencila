#include <stencila/stencil.hpp>
using namespace Stencila;

#include "extension.hpp"
#include "r-context.hpp"

STENCILA_R_NEW(Stencil)

STENCILA_R_EXEC1(Stencil,initialise,std::string)
STENCILA_R_EXEC1(Stencil,import,std::string)
STENCILA_R_EXEC1(Stencil,export_,std::string)
STENCILA_R_EXEC1(Stencil,read,std::string)
STENCILA_R_EXEC1(Stencil,write,std::string)

STENCILA_R_ATTR(Stencil,html,std::string)
STENCILA_R_ATTR(Stencil,cila,std::string)

STENCILA_R_RET0(Stencil,title)
STENCILA_R_RET0(Stencil,description)
STENCILA_R_RET0(Stencil,keywords)
STENCILA_R_RET0(Stencil,authors)

STENCILA_R_FUNC Stencil_attach(SEXP self,SEXP context){
    STENCILA_R_BEGIN
        RContext* rcontext = new RContext(context);
        from<Stencil>(self).attach(rcontext);
        return null;
    STENCILA_R_END
}
STENCILA_R_EXEC0(Stencil,detach)

STENCILA_R_RET0(Stencil,context) 

STENCILA_R_FUNC Stencil_render(SEXP self,SEXP context){
    STENCILA_R_BEGIN
    	if(context!=null){
	        RContext* rcontext = new RContext(context);
	        from<Stencil>(self).render(rcontext);
        } else {
        	from<Stencil>(self).render();
        }
        return null;
    STENCILA_R_END
}

STENCILA_R_RET0(Stencil,serve) 
STENCILA_R_EXEC0(Stencil,view)
