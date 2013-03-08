#include "r-workspace.hpp"

STENCILA_R_FUNC Workspace_test(void){
    STENCILA_R_BEGIN
        RWorkspace workspace;
        workspace.set("foo","'bar'");
        return wrap(workspace.text("foo"));
    STENCILA_R_END
}

