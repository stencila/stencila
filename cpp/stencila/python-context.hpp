#pragma once

#include <boost/python.hpp>

#include <stencila/context.hpp>
#include <stencila/exception.hpp>

namespace Stencila {

class PythonException : public Exception {
public:
    PythonException(std::string message): Exception(message) {}
};

/**
 * A `Context` for Python
 */
class PythonContext : public Context {
private:

    /**
     * Python code used to create an object representing this context on the Python side
     */
    static const char* code_(void){ 
        return
            #include "python-context.py"
        ;
    }

    /**
     * A boost::python object which represents this context on the Python "side"
     */
    boost::python::object context_;

    /**
     * Call a method of `context_`
     */
    template<typename... Args>
    boost::python::object call_(const char* name,Args... args){
        using namespace boost::python;
        try {
            return context_.attr(name)(args...);
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

    /**
     * Call a method of `context` and return the result
     */
    template<typename Result,typename... Args>
    Result call_(const char* name,Args... args){
        using namespace boost::python;
        return extract<Result>(call_(name,args...));
    }

public:

    PythonContext(void){
        using namespace boost::python;
        // Initialise the interpreter
        Py_Initialize();
        // Get the __main__ module's namespace
        object ns = import("__main__").attr("__dict__");
        // Execute the Python side code there
        exec(str(code_()),ns);
        // Create a new context
        context_ = ns["__stencila__context__"]();
    }
    
    PythonContext(boost::python::object context){
        context_ = context;
    }

    bool accept(const std::string& language) const {
        return language=="py";
    }

    void execute(const std::string& code){
        call_("execute",code);
    }

    std::string interact(const std::string& code){
        return call_<std::string>("interact",code);
    }

    void assign(const std::string& name, const std::string& expression){
        call_("assign",name,expression);
    }

    std::string write(const std::string& expression){
        return call_<std::string>("write",expression);
    }

    std::string paint(const std::string& format,const std::string& code){
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
        return call_<bool>("match",item,items);
    }

    bool next(void){
        return call_<bool>("next");
    }

    void enter(const std::string& expression=""){
        call_("enter",expression);
    }

    void exit(void){
        call_("exit");
    }
};

}
