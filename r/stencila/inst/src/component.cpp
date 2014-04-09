#include <stencila/component.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_NEW(Component)

STENCILA_R_ATTR(Component,title,std::string)
STENCILA_R_ATTR(Component,description,std::string)
STENCILA_R_ATTR(Component,keywords,std::vector<std::string>)
STENCILA_R_ATTR(Component,authors,std::vector<std::string>)

STENCILA_R_ATTR(Component,path,std::string)

STENCILA_R_EXEC1(Component,commit,std::string)

STENCILA_R_FUNC Component_log(SEXP self){
    STENCILA_R_BEGIN
    	// Get log
        auto log = from<Component>(self).log();
        // Convert to a data.frame
        uint rows = log.size();
        Rcpp::DatetimeVector time(rows);
        Rcpp::CharacterVector message(rows);
        Rcpp::CharacterVector name(rows);
        Rcpp::CharacterVector email(rows);
        for(uint i=0;i<rows;i++){
        	auto& commit = log[i];
        	time[i] = commit.time;
        	message[i] = commit.message;
        	name[i] = commit.name;
        	email[i] = commit.email;
        }
        return Rcpp::DataFrame::create(
        	Rcpp::Named("time") = time,
        	Rcpp::Named("message") = message,
        	Rcpp::Named("name") = name,
        	Rcpp::Named("email") = email,
        	// Don't convert strings to factors
        	Rcpp::Named("stringsAsFactors") = 0
        );
    STENCILA_R_END
}
