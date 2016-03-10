#include <stencila/component.hpp>
#include <stencila/stencil.hpp>
#include <stencila/sheet.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_NEW(Component)

STENCILA_R_GETSET(Component,path,std::string)

STENCILA_R_GET(Component,address)

STENCILA_R_GET(Component,held)

STENCILA_R_EXEC0(Component,vacuum)

STENCILA_R_GETSET(Component,managed,bool)

//STENCILA_R_GET(Component,publish)

STENCILA_R_GET(Component,origin)

STENCILA_R_EXEC0(Component,sync)

STENCILA_R_EXEC1(Component,commit,std::string)

STENCILA_R_FUNC Component_commits_get(SEXP self){
    STENCILA_R_BEGIN
    	// Get history
        auto commits = from<Component>(self).commits();
        // Convert to a data.frame
        unsigned int rows = commits.size();
        Rcpp::CharacterVector id(rows);
        Rcpp::DatetimeVector time(rows);
        Rcpp::CharacterVector message(rows);
        Rcpp::CharacterVector name(rows);
        Rcpp::CharacterVector email(rows);
        for(unsigned int i=0;i<rows;i++){
        	auto& commit = commits[i];
        	id[i] = commit.id;
            time[i] = commit.time;
        	message[i] = commit.message;
        	name[i] = commit.name;
        	email[i] = commit.email;
        }
        return Rcpp::DataFrame::create(
        	Rcpp::Named("id") = id,
            Rcpp::Named("time") = time,
        	Rcpp::Named("message") = message,
        	Rcpp::Named("name") = name,
        	Rcpp::Named("email") = email,
        	// Don't convert strings to factors
        	Rcpp::Named("stringsAsFactors") = 0
        );
    STENCILA_R_END
}

STENCILA_R_GET(Component,version)

STENCILA_R_EXEC2(Component,version,std::string,std::string)

STENCILA_R_GET(Component,versions)

STENCILA_R_GETSET(Component,branch,std::string)

STENCILA_R_GET(Component,branches)

STENCILA_R_EXEC2(Component,sprout,std::string,std::string)

STENCILA_R_EXEC2(Component,merge,std::string,std::string)

STENCILA_R_EXEC1(Component,lop,std::string)


Component* Component_instantiate(const std::string& address, const std::string& path, const std::string& type){
    Rcpp::Environment stencila("package:stencila");
    Rcpp::Function func = stencila["instantiate"];
    SEXP component = func(address, path, type);
    return &from<Component>(component);
}

STENCILA_R_FUNC Component_grab(SEXP address){
    STENCILA_R_BEGIN
        Component::Instance instance = Component::get(
            as<std::string>(address)
        );
        Component* component = instance.pointer();
        return Rcpp::List::create(
            Rcpp::Named("address") = component->address(),
            Rcpp::Named("path") = component->path(),
            Rcpp::Named("type") = instance.type_name()
        );
    STENCILA_R_END
}
