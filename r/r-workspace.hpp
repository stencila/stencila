#pragma once

#include <stencila/workspace.hpp>
using namespace Stencila;

#include "r-extension.hpp"

/*!
A specialisation of the Workspace class for R.

Implements the virtual methods of the Workspace class for the rendering of stencils in an
R environment. All the real functionality is done in an "R-side" Workspace class (see the R code)
and this class justs acts as a bridge between C++ and that code.

Uses the Rcpp::Language class which provides a much easier interface than using the 
'eval' R API function e.g. eval(name,R_GlobalEnv).
Note that although a function object has a () operator which calls a function in R I found difficulties
due to the way that it passes arguments (as a pairlist?). Using Rcpp::Language is more verbose but works.

There appear to  be several way to use Rcpp to get and call the R-side workspace method.
These include using the [] operator on the workspace and the () operator on a function object.
However, these don't always produce the expected results and so the best approach seems to be
to use the get() method, construct a Rcpp::Language object and then eval(). e.g.

    Rcpp::Language call(environment_.get("method_name"),arg1,arg2);
    call.eval();

Note that when the method is being called with no arguments it appear to be necessary to consturct a Rcpp::Function 
object first:

    Rcpp::Language call(Rcpp::Function(environment_.get("enter")));
    call.eval();

*/
class RWorkspace : public Workspace<RWorkspace> {
private:

    /*!
    An Rcpp object which represents workspace on the R "side"
    */
    Rcpp::Environment environment_;

public:

    static String type(void){
        return "r-workspace";
    };
    
    RWorkspace(void){
        Rcpp::Environment stencila("package:stencila");
        Rcpp::Function func = stencila.get("Workspace");
        Rcpp::Language call(func);
        environment_ = Rcpp::Environment(call.eval());
    }
    
    RWorkspace(const Id& id){
    }

    /*!
    Constructor which takes a SEXP representing the R-side Workspace.
    */
    RWorkspace(SEXP sexp){
        environment_ = Rcpp::Environment(sexp);
    }
    
    void read_from(const String& directory){
        Rcpp::Language call(Rcpp::Function(environment_.get("read_from")),directory);
        call.eval();
    }
    
    void write_to(const String& directory){
        Rcpp::Language call(Rcpp::Function(environment_.get("write_to")),directory);
        call.eval();
    }

    void set(const std::string& name, const std::string& expression){
        Rcpp::Language call(environment_.get("set"),name,expression);
        call.eval();
    }

    void script(const std::string& code){
        Rcpp::Language call(environment_.get("script"),code);
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

    void subject(const std::string& expression){
        Rcpp::Language call(environment_.get("subject"),expression);
        call.eval();
    }

    bool match(const std::string& expression){
        Rcpp::Language call(environment_.get("match"),expression);
        return as<bool>(call.eval());
    }

    void enter(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("enter")));
        call.eval();
    }

    void enter(const std::string& expression){
        Rcpp::Language call(environment_.get("enter"),expression);
        call.eval();
    }

    void exit(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("exit")));
        call.eval();
    }

    bool begin(const std::string& item,const std::string& items){
        Rcpp::Language call(environment_.get("begin"),item,items);
        return as<bool>(call.eval());
    }

    bool step(void){
        Rcpp::Language call(Rcpp::Function(environment_.get("step")));
        return as<bool>(call.eval());
    }
};
