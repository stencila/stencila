#pragma once

#include <stencila/contexts/context.hpp>
using namespace Stencila::Contexts;

#include "extension.hpp"

class PythonException : public Exception {
public:
    PythonException(std::string message): Exception(message) {}
};

/**
 * A specialisationof the COntext class for Python
 */
class PythonContext : public Context {
private:

    /*!
    A boost::python object which represents this context on the Python "side"
    */
    object object_;

    template<
        typename... Args
    >
    object call_(const char* name,Args... args){
        try {
            return object_.attr(name)(args...);
        }
        catch(error_already_set const &){
            PyObject *type,*value,*tb;
            PyErr_Fetch(&type,&value,&tb);
            handle<> hexc(type),hval(allow_null(value)),htb(allow_null(tb));

            std::string message = std::string(extract<std::string>(value))+":\n";

            object formatted_list = import("traceback").attr("format_exception")(hexc,hval,htb);
            
            for(int i=0;i<len(formatted_list);i++){
                message += std::string(extract<std::string>(formatted_list[i])) + "\n";
            }
            throw PythonException(message);
        }
    }

    template<
        typename Result,
        typename... Args
    >
    Result call_(const char* name,Args... args){
        return extract<Result>(call_(name,args...));
    }

public:
    
    PythonContext(object context){
       object_ = context;
    }

    std::string type(void) const {
        return "python-context";
    }


    void execute(const std::string& code){
        call_("execute",code);
    }


    std::string write(const std::string& expression){
        return call_<std::string>("write",expression);
    }
    

    std::string paint(const std::string& format,const std::string& code){
        return "";
    }
    

    bool test(const std::string& expression){
        return true;
    }


    void mark(const std::string& expression){

    }

    bool match(const std::string& expression){
        return true;
    }

    void unmark(void){

    }


    bool begin(const std::string& item,const std::string& items){
        return true;
    }

    bool next(void){
        return true;
    }


    void enter(const std::string& expression){

    }

    void enter(void){

    }

    void exit(void){

    }

};
