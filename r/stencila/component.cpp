#include <stencila/component.hpp>
using namespace Stencila;

#include "extension.hpp"

STENCILA_R_NEW(Component)

STENCILA_R_ATTR(Component,path,std::string)
STENCILA_R_RET0(Component,address)

STENCILA_R_RET0(Component,origin)

STENCILA_R_EXEC1(Component,commit,std::string)

STENCILA_R_FUNC Component_commits(SEXP self){
    STENCILA_R_BEGIN
    	// Get history
        auto commits = from<Component>(self).commits();
        // Convert to a data.frame
        uint rows = commits.size();
        Rcpp::DatetimeVector time(rows);
        Rcpp::CharacterVector message(rows);
        Rcpp::CharacterVector name(rows);
        Rcpp::CharacterVector email(rows);
        for(uint i=0;i<rows;i++){
        	auto& commit = commits[i];
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
