#include <string>

namespace Stencila {
namespace Python {
namespace Stencil_ {
        
PyObject* callback;

PyObject* set_callback(PyObject* callable){
    callback = callable;
    return Py_None;
}

void call_callback(void){
    //It seems that it is necessary to cal this at least once. See
    // http://stackoverflow.com/questions/4866701/python-pygilstate-ensure-release-causes-segfault-while-returning-to-c-from-p
    // http://stackoverflow.com/questions/7204664/pass-callback-from-python-to-c-using-boostpython
    //Perhaps this should only be called if PyEval_ThreadsInitialized return non-zero value. See
    // http://docs.python.org/c-api/init.html#PyEval_ThreadsInitialized
    PyEval_InitThreads();

    //Obtain the Python Global Interpreter Lock (GIL)
    PyGILState_STATE state = PyGILState_Ensure();
    //Call the Python callback function which should return a string
    std::string result = bp::call<std::string>(callback,"attr","value");
    //Release the GIL
    PyGILState_Release(state);
}
    
void bind(void){
    def("set_callback",set_callback);
    def("call_callback",call_callback);
}

}}}