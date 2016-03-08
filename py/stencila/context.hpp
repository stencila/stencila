#pragma once

#include <iostream>

#include <boost/python.hpp>
#include <boost/python/raw_function.hpp>

#include <stencila/context.hpp>
#include <stencila/exception.hpp>

// Experimental feature for embedding Python within a 
// binary for use outside of the Stencila Python package
// Currently, not fully tested nor used.
#define STENCILA_PY_EMBED 0

namespace Stencila {

class PythonException : public Exception {
public:
    PythonException(std::string message): Exception(message) {}
};

/**
 * A Python context
 *
 * Extends the abstract base class Context defined in `cpp/stencila/context.hpp`
 */
class PythonContext : public Context {
public:
    
    PythonContext(boost::python::object context){
        // This seems to fix #71 and is based on this answer http://stackoverflow.com/a/12118259/4625911
        //      In general situations, the C library needs to call PyEval_InitThreads() 
        //      to gain GIL before spawning any thread that invokes python callbacks. 
        //      And the callbacks need to be surrounded with PyGILState_Ensure() and 
        //      PyGILState_Release() to ensure safe execution.
        PyEval_InitThreads();

        context_ = context;
    }

    #if STENCILA_PY_EMBED
    PythonContext(void){
        using namespace boost::python;
        // Initialise the interpreter
        Py_Initialize();
        // Get the __main__ module's namespace
        object main = import("__main__");
        object globals = main.attr("__dict__");
        // Execute the Python-side in the glabals namespace
        exec(str(code_()),globals);
        // Create a new Python-side Context
        context_ = globals["Context"]();
        // Bind to the callback
        context_.attr("bind")(raw_function(callback_));
    }
    #endif

    /**
     * Implementation of pure virtual functions defined by `Context` class.
     * see there for documentation.
     */

    std::string details(void) const {
        return "PythonContext at " + string(this);
    };

    bool accept(const std::string& language) const {
        return language=="py";
    }

    std::string execute(
        const std::string& code, 
        const std::string& id="", 
        const std::string& format="", 
        const std::string& width="", 
        const std::string& height="", 
        const std::string& units=""
    ){
        return get_<std::string>("execute",code,id,format,width,height,units);
    }

    std::string interact(const std::string& code, const std::string& id=""){
        return get_<std::string>("interact",code);
    }

    void assign(const std::string& name, const std::string& expression){
        call_("assign",name,expression);
    }

    void input(const std::string& name, const std::string& type, const std::string& value){
        call_("input",name,type,value);
    };

    std::string write(const std::string& expression){
        return get_<std::string>("write",expression);
    }

    std::string paint(const std::string& format,const std::string& code){
        return get_<std::string>("paint",format,code);
    }

    bool test(const std::string& expression){
        return get_<bool>("test",expression);
    }

    void mark(const std::string& expression){
        call_("mark",expression);
    }

    bool match(const std::string& expression){
        return get_<bool>("match",expression);
    }

    void unmark(void){
        call_("unmark");
    }

    bool begin(const std::string& item,const std::string& items){
        return get_<bool>("begin",item,items);
    }

    bool next(void){
        return get_<bool>("next");
    }

    void enter(const std::string& expression=""){
        call_("enter",expression);
    }

    void exit(void){
        call_("exit");
    }

    /**
     * Register the `PythonContext` class
     */
    static void class_(void){
        Class::set(PythonContextType,{
            "PythonContext"
        });
    }

    /**
     * Serve this context
     */
    std::string serve(void){
        return Component::serve(PythonContextType);
    }

    /**
     * View this context
     */
    PythonContext& view(void){
        Component::view(PythonContextType);
        return *this;
    }

private:
    /**
     * A boost::python object which represents this context on the Python "side"
     */
    boost::python::object context_;

    #ifdef STENCILA_PY_EMBED
    /**
     * Python code used to create an object representing this context on the Python side
     */
    static const char* code_(void){ 
        return "Code in context.py should be returned to be compiled into binary";
    }
    #endif

    /**
     * Call a method of `context_`
     */
    template<typename... Args>
    boost::python::object call_(const char* name,Args... args){
        using namespace boost::python;

        // Get the Python GIL (Global Interpreter Lock). Ensure it
        // is released for any of the branches below.
        PyGILState_STATE py_gil_state = PyGILState_Ensure();
        try {
            // Call the Python side context method
            auto method = context_.attr(name);      
            auto result = method(args...);

            // Release the GIL
            PyGILState_Release(py_gil_state);            
            return result;
        }
        catch(error_already_set const &){
            // Get the error
            PyObject *type,*value,*traceback;
            PyErr_Fetch(&type,&value,&traceback);
            // Construct a message
            std::string message;
            if(value){
                extract<std::string> value_string(value);
                // Check a string can be extracted from the PyObject
                if(value_string.check()){
                    message += value_string() +":\n";
                }
            }
            // There may not be a traceback (e.g. if a syntax error)
            if(value and traceback){
                handle<> type_handle(type);
                handle<> value_handle(allow_null(value));
                handle<> traceback_handle(allow_null(traceback));
                object formatted_list = import("traceback").attr("format_exception")(type_handle,value_handle,traceback_handle);
                for(int i=0;i<len(formatted_list);i++){
                    extract<std::string> line_string(formatted_list[i]);
                    // Check a string can be extracted from the PyObject
                    if(line_string.check()){
                        message += line_string();
                    }
                }
            }

            // Release the GIL
            PyGILState_Release(py_gil_state);
            throw PythonException(message);
        }
        catch(const std::exception& exc){
            // Release the GIL
            PyGILState_Release(py_gil_state);
            throw PythonException(exc.what());
        }
        catch(...){
            // Release the GIL
            PyGILState_Release(py_gil_state);
            throw PythonException("Unknown exception");
        }
    }

    /**
     * Call a method of `context_` and return the result
     */
    template<typename Result,typename... Args>
    Result get_(const char* name,Args... args){
        using namespace boost::python;
        auto result = call_(name,args...);
        return extract<Result>(result);
    }

    // Experimental; not currently used
    #if 0
    /**
     * Call a method of this context from Python.
     *
     * Note that self must be provided as an argument
     */
     /*
     There are a couple of way to do this including using boost::bind to bind instance of context so a particular
     method can be called e.g. 

        boost::function<std::string (const char*)> some_method_binding( boost::bind( &PythonContext::some_method, this, _1 ) );
        ...
        make_function(some_method_binding,default_call_policies(),boost::mpl::vector<std::string, const char*>())

      */
    static boost::python::object callback_(boost::python::tuple args, boost::python::dict kwargs){  
        return boost::python::object();
    }
    #endif

};

}
