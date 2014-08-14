#include <stencila/stencil.hpp>

// Conditional includes of context types
#if STENCILA_PYTHON_CONTEXT
    #include <stencila/python-context.hpp>
#endif
#if STENCILA_R_CONTEXT
    #include <stencila/r-context.hpp>
#endif

namespace Stencila {

Stencil& Stencil::render(const std::string& type){
    // Get the list of context that are compatible with this stencil
    auto types = contexts();
    // Use the first in the list if type has not been specified
    std::string use;
    if(type.length()==0){
        if(types.size()==0){
            STENCILA_THROW(Exception,"No default context type for this stencil; please specify one.");
        }
        else use = types[0];
    } else {
        use = type;
    }
    // Render the stencil in the corresponding context type
    if(use=="py"){
        #if STENCILA_PYTHON_CONTEXT
            PythonContext context;
            render(context);
        #else
            STENCILA_THROW(Exception,"Stencila has not been compiled with support for Python contexts");
        #endif
    }
    else if(use=="r"){
        #if STENCILA_R_CONTEXT
            RContext context;
            render(context);
        #else
            STENCILA_THROW(Exception,"Stencila has not been compiled with support for R contexts");
        #endif
    }
    else {
       STENCILA_THROW(Exception,"Unrecognised context type: "+type); 
    }
    // Return self for chaining
    return *this;
}

Stencil& Stencil::render(void){
    return render(std::string());
}

} // namespace Stencila
