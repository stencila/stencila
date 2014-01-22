#include <stencila/exception.hpp>
using namespace Stencila;

#include "extension.hpp"

template<typename Class>
struct ExceptionTranslator {
    static PyObject* type;
    static void translate(const Class& exception){
    	PyErr_SetObject(type, object(exception).ptr());
    }
};

template<typename Class>
PyObject* ExceptionTranslator<Class>::type;

void exception_test(void){
	throw Exception("Testing, testing, 1, 2, 3.");
}

// For the line
// 		klass.def(str(self));
// below it is necessary to do the following
namespace boost {
namespace python {
    using self_ns::str;
}
}

void def_Exception(void){

    class_<Exception> klass("Exception");
    klass.def(str(self));
    ExceptionTranslator<Exception>::type = klass.ptr();
    register_exception_translator<Exception>(ExceptionTranslator<Exception>::translate);

    def("exception_test",exception_test);
}
