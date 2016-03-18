#include <boost/python.hpp>
#include <boost/python/raw_function.hpp>

#include <stencila/spread.hpp>
#include <stencila/function.hpp>

namespace Stencila {

class PythonSpread : public Spread {
public:

    PythonSpread(boost::python::object spread) {
        spread_ = spread;
        PyEval_InitThreads();
    }

    virtual ~PythonSpread(void) {
    }

    /**
     * @name Spread interface implementation
     *
     * For documentation on these methods see the base abstract class `Spread`
     *
     * @{
     */
    
    std::string execute(const std::string& source) {
        return call_<std::string>("execute", source);
    }

    std::string evaluate(const std::string& expression) {
        return call_<std::string>("evaluate", expression);
    }

    std::string set(const std::string& id, const std::string& expression, const std::string& name = "") {
        return call_<std::string>("set", id, expression, name);
    }

    std::string get(const std::string& name) {
        return call_<std::string>("get", name);
    }

    std::string clear(const std::string& id = "", const std::string& name = "") {
        return call_<std::string>("clear", id, name);
    }

    std::string list(void) {
        return call_<std::string>("list");
    }

    std::string collect(const std::vector<std::string>& cells) {
        return "[" + join(cells, ",") + "]";
    }

    std::string depends(const std::string& expression) {
        return call_<std::string>("depends", expression);
    }

    std::vector<std::string> functions(void) {
        // TODO
        return {};
    }

    Function function(const std::string& name) {
        // TODO
        return Function();
    }

    void read(const std::string& path) {
        // TODO
    }

    void write(const std::string& path) {
        // TODO
    }

    /**
     * @}
     */

private:
    /**
     * A boost::python object which represents this spread on the Python side
     */
    boost::python::object spread_;

    /**
     * Call a method on the Python side spread
     *
     * This method is copied from `PythonContext::call_` (with tidy-ups!) : this code
     * should probably be shared somehow (e.g. by having a common base class)
     */
    template<typename... Args>
    boost::python::object call_(const char* name, Args... args) {
        using namespace boost::python;

        // Get the Python GIL (Global Interpreter Lock). Ensure it
        // is released for any of the branches below.
        PyGILState_STATE py_gil_state = PyGILState_Ensure();
        try {
            // Call the Python side context method
            auto method = spread_.attr(name);
            auto result = method(args...);

            // Release the GIL
            PyGILState_Release(py_gil_state);  
            return result;
        }
        catch(error_already_set const &) {
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
                object formatted_list = boost::python::import("traceback").attr("format_exception")(type_handle,value_handle,traceback_handle);
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
            STENCILA_THROW(Exception, message);
        }
        catch(const std::exception& exc) {
            // Release the GIL
            PyGILState_Release(py_gil_state);
            STENCILA_THROW(Exception, exc.what());
        }
        catch(...) {
            // Release the GIL
            PyGILState_Release(py_gil_state);
            STENCILA_THROW(Exception, "Unknown exception");
        }
    }

    /**
     * Call a method on the Python side and get the return value
     */
    template<typename Result, typename... Args>
    Result call_(const char* name, Args... args){
        auto result = call_(name,args...);
        return  boost::python::extract<Result>(result);
    }

};

}
