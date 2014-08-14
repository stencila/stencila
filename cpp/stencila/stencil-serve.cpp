#include <stencila/stencil.hpp>

namespace Stencila {

std::string serve(void){
    return Component::serve(StencilCode);
}

void view(void){
    return Component::view(StencilCode);
}

std::string call(const Call& call) {
    auto what = call.what();
    
    // Getting content
    if(what=="html():string"){
        return html();
    }
    else if(what=="cila():string"){
        return cila();
    }

    // Setting content
    else if(what=="html(string)"){
        std::string string = call.arg(0);
        html(string);
    }
    else if(what=="cila(string)"){
        std::string string = call.arg(0);
        cila(string);
    }

    // Conversion of content...
    // ... HTML to Cila
    else if(what=="html(string).cila():string"){
        std::string string = call.arg(0);
        return     html(string).cila();
    }
    // ... Cila to HTML
    else if(what=="cila(string).html():string"){
        std::string string = call.arg(0);
        return     cila(string).html();
    }

    // Rendering...
    //... HTML
    else if(what=="html(string).render().html():string"){
        std::string string = call.arg(0);
        return     html(string).render().html();
    }
    //...Cila
    else if(what=="cila(string).render().cila():string"){
        std::string string = call.arg(0);
        return     cila(string).render().cila();
    }

    else return Component::call(call);

    return "";
}

static std::string page(const Component* component){
    return static_cast<const Stencil&>(*component).dump();
}

static std::string call(Component* component, const Call& call){
    return static_cast<Stencil&>(*component).call(call);
}

}
