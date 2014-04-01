#pragma once

#include <stencila/contexts/context.hpp>
using namespace Stencila::Contexts;

#include "stencila.hpp"

/*!
A specialisation of the Context class for R.

Implements the virtual methods of the Context class for the rendering of stencils in an
R environment. All the real functionality is done in an "R-side" Context class (see the R code)
and this class justs acts as a bridge between C++ and that code.

Uses the Rcpp::Language class which provides a much easier interface than using the 
'eval' R API function e.g. eval(name,R_GlobalEnv).
Note that although a function object has a () operator which calls a function in R I found difficulties
due to the way that it passes arguments (as a pairlist?). Using Rcpp::Language is more verbose but works.

There appear to  be several way to use Rcpp to get and call the R-side context method.
These include using the [] operator on the context and the () operator on a function object.
However, these don't always produce the expected results and so the best approach seems to be
to use the get() method, construct a Rcpp::Language object and then eval(). e.g.

    Rcpp::Language call(environment_.get("method_name"),arg1,arg2);
    call.eval();

Note that when the method is being called with no arguments it appear to be necessary to consturct a Rcpp::Function 
object first:

    Rcpp::Language call(Rcpp::Function(environment_.get("enter")));
    call.eval();

*/
class RContext : public Context<RContext> {
private:

    /*!
    An Rcpp object which represents context on the R "side"
    */
    Rcpp::Environment environment_;

public:
    
    RContext(void){
        Rcpp::Environment stencila("package:stencila");
        Rcpp::Function func = stencila.get("Context");
        Rcpp::Language call(func);
        environment_ = Rcpp::Environment(call.eval());
    }

    /*!
    Constructor which takes a SEXP representing the R-side Context.
    */
    RContext(SEXP sexp){
        environment_ = Rcpp::Environment(sexp);
    }

    std::string type(void) const {
        return "r-context";
    }
    

    void read_from(const std::string& directory){
        Rcpp::Language call(Rcpp::Function(environment_.get("read_from")),directory);
        call.eval();
    }
    
    void write_to(const std::string& directory){
        Rcpp::Language call(Rcpp::Function(environment_.get("write_to")),directory);
        call.eval();
    }


    void set(const std::string& name, const std::string& expression){
        Rcpp::Language call(environment_.get("set"),name,expression);
        call.eval();
    }


    void execute(const std::string& code){
        Rcpp::Language call(environment_.get("execute"),code);
        call.eval();
    }
    

    std::string interact(const std::string& code){
        Rcpp::Language call(environment_.get("interact"),code);
        return as<std::string>(call.eval());
    }


    std::string text(const std::string& expression){
        Rcpp::Language call(environment_.get("text"),expression);
        return as<std::string>(call.eval());
    }
    

    void image_begin(const std::string& type){
        Rcpp::Language call(environment_.get("image_begin"),type);
        call.eval();
    }
    
    std::string image_end(){
        Rcpp::Language call(Rcpp::Function(environment_.get("image_end")));
        return as<std::string>(call.eval());
    }


    bool test(const std::string& expression){
        Rcpp::Language call(environment_.get("test"),expression);
        return as<bool>(call.eval());
    }


    void mark(const std::string& expression){
        Rcpp::Language call(environment_.get("mark"),expression);
        call.eval();
    }

    bool match(const std::string& expression){
        Rcpp::Language call(environment_.get("match"),expression);
        return as<bool>(call.eval());
    }

    void unmark(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("unmark")));
        call.eval();
    }


    bool begin(const std::string& item,const std::string& items){
        Rcpp::Language call(environment_.get("begin"),item,items);
        return as<bool>(call.eval());
    }

    bool next(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("next_")));
        return as<bool>(call.eval());
    }


    void enter(const std::string& expression){
        Rcpp::Language call(environment_.get("enter"),expression);
        call.eval();
    }

    void enter(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("enter")));
        call.eval();
    }

    void exit(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("exit")));
        call.eval();
    }

};
