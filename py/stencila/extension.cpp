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
void def_Stencil(void);
void def_Theme(void);

BOOST_PYTHON_MODULE(extension){
	// Declare converters
	to_python_converter<std::vector<std::string>, vector_to_list<std::string>>();
	// Define classes
	def_Exception();
	def_Component();
    def_Stencil();
    def_Theme();
}
