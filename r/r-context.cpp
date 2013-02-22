#include "r-context.hpp"

STENCILA_R_FUNC Context_test(void){
    STENCILA_R_BEGIN
        RContext context;
        context.set("foo","'bar'");
        return wrap(context.text("foo"));
    STENCILA_R_END
}

