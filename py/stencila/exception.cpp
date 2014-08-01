#include <stencila/exception.hpp>

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

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
	throw Stencila::Exception("Testing, testing, 1, 2, 3.");
}

void def_Exception(void){
    class_<Exception> klass("Exception");
    //klass.def(str(self));
    ExceptionTranslator<Exception>::type = klass.ptr();
    register_exception_translator<Exception>(ExceptionTranslator<Exception>::translate);

    def("exception_test",exception_test);
}
