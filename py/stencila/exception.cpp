#pragma once

#include "../../cpp/exception.hpp"

namespace Stencila {
namespace Python {
namespace ExceptionBindings {
	
using namespace Stencila;
		
template<typename Class>
struct ExceptionTranslator {
	static PyObject* type;
	static void translate(const Class& exception){
		PyErr_SetObject(type, object(exception).ptr());
	}
};
template<typename Class>
PyObject* ExceptionTranslator<Class>::type;

void bind(void){
	class_<Exception> klass("Exception");
	klass.def(str(self));
	ExceptionTranslator<Exception>::type = klass.ptr();
	register_exception_translator<Exception>(ExceptionTranslator<Exception>::translate);
}

}}}