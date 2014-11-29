#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::serve(void){
    return Component::serve(StencilType);
}

void Stencil::view(void){
    return Component::view(StencilType);
}

std::string Stencil::call(const Call& call) {
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
    //... Cila
    else if(what=="cila(string).render().cila():string"){
        std::string string = call.arg(0);
        return     cila(string).render().cila();
    }
    //... update <input>s
    else if(what=="inputs({string,string}).render().html():string"){
        auto values = call.arg<std::map<std::string,std::string>>(0);
        return     inputs(     values    ).render().html();
    }
    //... restart
    else if(what=="restart().html():string"){
        return     restart().html();
    }    

    // Access to context
    else if(what=="context.interact(string):string"){
        if(context_){
            std::string string = call.arg(0);
            return     context_->interact(string);
        } else {
            STENCILA_THROW(Exception,"No context attached to this stencil");
        }
    }

    else return Component::call(call);

    return "";
}

std::string Stencil::page(const Component* component){
    // Retun HTML for a complete HTML document with indentation
    return static_cast<const Stencil&>(*component).html(true,true);
}

std::string Stencil::call(Component* component, const Call& call){
    return static_cast<Stencil&>(*component).call(call);
}

}
