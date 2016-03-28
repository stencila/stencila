#include <string>
#include <vector>

#include <stencila/component.hpp>
#include <stencila/exception.hpp>

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

Component* Component_instantiate(const std::string& address, const std::string& path, const std::string& type);

// Define converters
template<typename Type>
struct vector_to_list {
	static PyObject* convert(const std::vector<Type>& vec) {
		list* l = new list();
		for(size_t i = 0; i < vec.size(); i++) (*l).append(vec[i]);
		return l->ptr();
	}
};

// Define exception translation
void exception_translator(const Exception& exception){
	PyErr_SetString(PyExc_RuntimeError, exception.what());
}
void exception_test(void){
	throw Exception("Testing, testing, 1, 2, 3.");
}

// Forward declarations of functions defined in other
// source files
void def_Component(void);
void def_Stencil(void);
void def_Theme(void);
void def_Sheet(void);

BOOST_PYTHON_MODULE(extension){
	// Declare converters
	to_python_converter<std::vector<std::string>, vector_to_list<std::string>>();

	// Declare exception translation
	register_exception_translator<Exception>(exception_translator);
    def("exception_test",exception_test);

	// Define component classes
	def_Component();
    def_Stencil();
    def_Theme();
    def_Sheet();

    // Declare component class types
    Component::classes();

    // Define the instantiation function
    Component::instantiate = Component_instantiate;
}
