#include <stencila/http-server.hpp>
using Stencila::Http::Server;

#include "r-extension.hpp"

//For turning off R stack limit checking (see Server_start() below)
#define CSTACK_DEFNS 7
#define HAVE_UINTPTR_T
#include <Rinterface.h>
//For restoring stack limit checking (see Server_start() below)
uintptr_t RStackLimit;

STENCILA_R_FUNC HttpServer_new(void){
    STENCILA_R_BEGIN
        return STENCILA_R_TO(Server,new Server);
    STENCILA_R_END
}

STENCILA_R_FUNC HttpServer_start(SEXP self){
    STENCILA_R_BEGIN
        //Turn off R stack limit checking
        //This is the only workaround I could find to "Error: C stack usage is too close to the limit"
        //which was raised because Server::start() starts a new thread. 
        //As [Writing R Extensions](http://cran.r-project.org/doc/manuals/R-exts.html) says "code which makes use of the stack-checking mechanism must not be called from threaded code.".
        //See also
        //  http://stackoverflow.com/questions/14719349/error-c-stack-usage-is-too-close-to-the-limit
        //  https://stat.ethz.ch/pipermail/r-sig-mac/2012-July/009336.html
        RStackLimit = R_CStackLimit;
        R_CStackLimit=(uintptr_t)-1;
        from<Server>(self).start();
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC HttpServer_stop(SEXP self){
    STENCILA_R_BEGIN
        from<Server>(self).stop();
        R_CStackLimit = RStackLimit;
        return nil;
    STENCILA_R_END
}

STENCILA_R_FUNC HttpServer_run(SEXP self){
    STENCILA_R_BEGIN
        from<Server>(self).run();
        return nil;
    STENCILA_R_END
}
