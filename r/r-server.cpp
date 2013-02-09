#include <stencila/server.hpp>
using Stencila::Server;

#include "r-extension.hpp"

STENCILA_R_FUNC Server_new(void){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Server,new Server);
    STENCILA_R_END
}

STENCILA_R_FUNC Server_start(SEXP self){
    STENCILA_R_BEGIN
        from<Server>(self).start();
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC Server_stop(SEXP self){
    STENCILA_R_BEGIN
        from<Server>(self).stop();
        return nil;
    STENCILA_R_END
}
