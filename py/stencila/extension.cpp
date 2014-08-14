#include <string>
#include <vector>

#include <boost/python.hpp>

using namespace boost::python;

// Define converters
template<typename Type>
struct vector_to_list {
	static PyObject* convert(const std::vector<Type>& vec) {
		list* l = new list();
		for(size_t i = 0; i < vec.size(); i++) (*l).append(vec[i]);
		return l->ptr();
	}
};

// Forward declarations of functions defined in other
// source files
void def_Exception(void);
void def_Component(void);
void def_Package(void);
void def_Stencil(void);

BOOST_PYTHON_MODULE(extension){
	// Declare converters
	to_python_converter<std::vector<std::string>, vector_to_list<std::string>>();

	// Declare exception translation and general Stencila functions
	def_Exception();

	// Declare classes
	def_Component();
    def_Package();
    def_Stencil();
}
