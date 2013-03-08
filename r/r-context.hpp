#pragma once

#include <stencila/context.hpp>
using namespace Stencila;

#include "r-extension.hpp"

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

    Rcpp::Language call(context.get("method_name"),arg1,arg2);
    call.eval();

Note that when the method is being called with no arguments it appear to be necessary to consturct a Rcpp::Function 
object first:

    Rcpp::Language call(Rcpp::Function(context.get("enter")));
    call.eval();

*/
class RContext : public Context<RContext> {
private:

    /*!
    An Rcpp object which represents context on the R "side"
    */
    Rcpp::Environment context;

public:

    static String type(void){
        return "r-context";
    };
    
    RContext(void){
        Rcpp::Environment stencila("package:stencila");
        Rcpp::Function func = stencila.get("Context");
        Rcpp::Language call(func);
        context = Rcpp::Environment(call.eval());
    }
    
    RContext(const Id& id){
    }

    /*!
    Constructor which takes a SEXP representing the R-side Context.
    */
    RContext(SEXP sexp){
        context = Rcpp::Environment(sexp);
    }
    
    void read_from(const String& directory){
        Rcpp::Language call(Rcpp::Function(context.get("read_from")),directory);
        call.eval();
    }
    
    void write_to(const String& directory){
        Rcpp::Language call(Rcpp::Function(context.get("write_to")),directory);
        call.eval();
    }

    void set(const std::string& name, const std::string& expression){
        Rcpp::Language call(context.get("set"),name,expression);
        call.eval();
    }

    void script(const std::string& code){
        Rcpp::Language call(context.get("script"),code);
        call.eval();
    }
    
    std::string interact(const std::string& code){
        Rcpp::Language call(context.get("interact"),code);
        return as<std::string>(call.eval());
    }

    std::string text(const std::string& expression){
        Rcpp::Language call(context.get("text"),expression);
        return as<std::string>(call.eval());
    }

    bool test(const std::string& expression){
        Rcpp::Language call(context.get("test"),expression);
        return as<bool>(call.eval());
    }

    void subject(const std::string& expression){
        Rcpp::Language call(context.get("subject"),expression);
        call.eval();
    }

    bool match(const std::string& expression){
        Rcpp::Language call(context.get("match"),expression);
        return as<bool>(call.eval());
    }

    void enter(void){
        Rcpp::Language call(Rcpp::Function(context.get("enter")));
        call.eval();
    }

    void enter(const std::string& expression){
        Rcpp::Language call(context.get("enter"),expression);
        call.eval();
    }

    void exit(void){
        Rcpp::Language call(Rcpp::Function(context.get("exit")));
        call.eval();
    }

    bool begin(const std::string& item,const std::string& items){
        Rcpp::Language call(context.get("begin"),item,items);
        return as<bool>(call.eval());
    }

    bool step(void){
        Rcpp::Language call(Rcpp::Function(context.get("step")));
        return as<bool>(call.eval());
    }
};
