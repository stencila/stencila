#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::serve(void){
    return Component::serve(StencilType);
}

void Stencil::view(void){
    return Component::view(StencilType);
}

std::string Stencil::interact(const std::string& code){
    if(context_){
        // Switch to stencil's directory
        boost::filesystem::path cwd = boost::filesystem::current_path();
        boost::filesystem::path path = boost::filesystem::path(Component::path(true)); 
        boost::filesystem::current_path(path);
        // Create a new unique id
        static char chars[] = {
            'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z',
            'A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
            '0','1','2','3','4','5','6','7','8','9'
        };
        std::string id;
        for(int cha=0;cha<8;cha++) id += chars[int(random()/double(RAND_MAX)*sizeof(chars))];
        // Run code in context
        auto result = context_->interact(code,id);
        // Return to original working directory
        boost::filesystem::current_path(cwd);
        return result;
    } else {
        STENCILA_THROW(Exception,"No context attached to this stencil");
    }
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
    else if(what=="interact(string):string"){
        std::string string = call.arg(0);
        return     interact(string);
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
