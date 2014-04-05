#pragma once

#include <stencila/contexts/context.hpp>
using namespace Stencila::Contexts;

#include "stencila.hpp"

/*!
A specialisation of the Context class for R.

Implements the methods of the Context class for the rendering of stencils in an
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
class RContext : public Context {
private:

    /*!
    An Rcpp object which represents this context on the R "side"
    */
    Rcpp::Environment environment_;

    template<
        typename... Args
    >
    SEXP call_(const char* name,Args... args){
        Rcpp::Language call(Rcpp::Function(environment_.get(name)),args...);
        return call.eval();
    }

    template<
        typename Result,
        typename... Args
    >
    Result call_(const char* name,Args... args){
        return as<Result>(call_(name,args...));
    }

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

    void execute(const std::string& code){
        call_("execute",code);
    }

    void assign(const std::string& name, const std::string& expression){
        call_("set",name,expression);
    }

    std::string write(const std::string& expression){
        return call_<std::string>("write",expression);
    }
    
    std::string paint(const std::string& format, const std::string& code){
        return call_<std::string>("paint",format,code);
    }
    
    bool test(const std::string& expression){
        return call_<bool>("test",expression);
    }

    void mark(const std::string& expression){
        call_("mark",expression);
    }

    bool match(const std::string& expression){
        return call_<bool>("match",expression);
    }

    void unmark(void){
        call_("unmark");
    }

    bool begin(const std::string& item,const std::string& items){
        return call_<bool>("begin",item,items);
    }

    bool next(void){
        return call_<bool>("next_");
    }

    void enter(const std::string& expression){
        call_("enter",expression);
    }

    void enter(void){
        call_("enter");
    }

    void exit(void){
        call_("exit");
    }

};
