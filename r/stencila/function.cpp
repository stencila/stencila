#include <stencila/function.hpp>
using namespace Stencila;

#include "stencila.hpp"
#include "context.hpp"

// Component interface

STENCILA_R_NEW(Function)

STENCILA_R_EXEC1(Function,initialise,std::string)

STENCILA_R_EXEC2(Function,load,std::string,std::string)
STENCILA_R_RET1(Function,dump,std::string)

STENCILA_R_EXEC1(Function,import,std::string)
STENCILA_R_FUNC Function_export(SEXP self,SEXP path){
    STENCILA_R_BEGIN
        from<Function>(self).export_(as<std::string>(path));
        return null;
    STENCILA_R_END
}

STENCILA_R_EXEC1(Function,read,std::string)
STENCILA_R_EXEC1(Function,write,std::string)

STENCILA_R_RET0(Function,serve) 
STENCILA_R_EXEC0(Function,view)

// ExecutableComponent interface

STENCILA_R_FUNC Function_attach(SEXP self,SEXP context){
    STENCILA_R_BEGIN
        from<Function>(self).attach(std::make_shared<RContext>(context));
        return null;
    STENCILA_R_END
}

// Function interface

STENCILA_R_FUNC Function_rd_set(SEXP self, SEXP rd){
    STENCILA_R_BEGIN
        Function& func = from<Function>(self);
        Rcpp::List list = as<Rcpp::List>(rd);
        auto name = as<std::string>(list["name"]);
        func.name(name);
        auto title = as<std::string>(list["title"]);
        func.title(title);
        auto summary = as<std::string>(list["summary"]);
        func.summary(summary);
        auto details = as<std::string>(list["details"]);
        func.details(details);

        auto parameters = as<Rcpp::List>(list["parameters"]);
        for(Rcpp::List::iterator iter = parameters.begin(); iter != parameters.end(); ++iter) {
            auto parameter = as<Rcpp::List>(*iter);
            auto name = as<std::string>(parameter["arg"]);
            auto description = as<std::string>(parameter["description"]);
            func.parameter({name,description});
        }
        return null;
    STENCILA_R_END
}

STENCILA_R_RET0(Function,json)

//STENCILA_R_RET1(Function,name,std::string)

//STENCILA_R_RET0(Function,call) 
