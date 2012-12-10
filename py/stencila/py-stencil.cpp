#include <string>

#include <stencila/stencil.hpp>
using namespace Stencila;

#include "py-extension.hpp"

PyObject* Stencil_callback;

PyObject* Stencil_set_callback(PyObject* callable){
    Stencil_callback = callable;
    return Py_None;
}

void Stencil_call_callback(void){
    //It seems that it is necessary to call PyEval_InitThreads() at least once. See
    //  http://stackoverflow.com/questions/4866701/python-pygilstate-ensure-release-causes-segfault-while-returning-to-c-from-p
    //  http://stackoverflow.com/questions/7204664/pass-callback-from-python-to-c-using-boostpython
    //Perhaps this should only be called if PyEval_ThreadsInitialized return non-zero value. See
    //  http://docs.python.org/c-api/init.html#PyEval_ThreadsInitialized
    PyEval_InitThreads();

    //Obtain the Python Global Interpreter Lock (GIL)
    PyGILState_STATE state = PyGILState_Ensure();
    //Call the Python callback function which should return a string
    std::string result = bp::call<std::string>(Stencil_callback,"attr","value");
    //Release the GIL
    PyGILState_Release(state);
}
    
void Stencil_define(void){
    def("set_callback",Stencil_set_callback);
    def("call_callback",Stencil_call_callback);
}
