#pragma once

#include <vector>
#include <string>

#include <stencila/spread.hpp>
#include <stencila/function.hpp>

#include "extension.hpp"

namespace Stencila {

class PythonSpread : public Spread {
 public:
    explicit PythonSpread(boost::python::object spread) {
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
        return call_("execute", source);
    }

    std::string evaluate(const std::string& expression) {
        return call_("evaluate", expression);
    }

    std::string set(const std::string& id, const std::string& expression, const std::string& name = "") {
        return call_("set", id, expression, name);
    }

    std::string get(const std::string& name) {
        return call_("get", name);
    }

    std::string clear(const std::string& id = "", const std::string& name = "") {
        return call_("clear", id, name);
    }

    std::string list(void) {
        return call_("list");
    }

    std::string collect(const std::vector<std::string>& cells) {
        return "[" + join(cells, ",") + "]";
    }

    std::string depends(const std::string& expression) {
        return call_("depends", expression);
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
        call_("read", path);
    }

    void write(const std::string& path) {
        call_("write", path);
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
    template<
        typename Result = std::string,
        typename... Args
    >
    Result call_(const char* name, Args... args) {
        using namespace boost::python;
        object result;

        // Get the Python GIL (Global Interpreter Lock). Ensure it
        // is released for any of the branches below.
        PyGILState_STATE py_gil_state = PyGILState_Ensure();
        try {
            // Call the Python side context method
            auto method = spread_.attr(name);
            result = method(args...);

            // Release the GIL
            PyGILState_Release(py_gil_state);
        }
        catch(error_already_set const &) {
            // Get the error
            PyObject *type, *value, *traceback;
            PyErr_Fetch(&type, &value, &traceback);
            // Construct a message
            std::string message;
            if (value) {
                extract<std::string> value_string(value);
                // Check a string can be extracted from the PyObject
                if (value_string.check()) {
                    message += value_string() +":\n";
                }
            }
            // There may not be a traceback (e.g. if a syntax error)
            if (value and traceback) {
                handle<> type_handle(type);
                handle<> value_handle(allow_null(value));
                handle<> traceback_handle(allow_null(traceback));
                object formatted_list = import("traceback").attr("format_exception")(
                    type_handle, value_handle, traceback_handle
                );
                for (int i = 0; i < len(formatted_list); i++) {
                    extract<std::string> line_string(formatted_list[i]);
                    // Check a string can be extracted from the PyObject
                    if (line_string.check()) {
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

        return  extract<Result>(result);
    }
};

}
