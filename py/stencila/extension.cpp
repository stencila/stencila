#include <vector>

#include <stencila/version.hpp>

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

void def_Exception(void);
void def_Component(void);
void def_Package(void);
void def_Stencil(void);

// Define converters
template<typename Type>
struct vector_to_list {
	static PyObject* convert(const std::vector<Type>& vec) {
		list* l = new list();
		for(size_t i = 0; i < vec.size(); i++) (*l).append(vec[i]);
		return l->ptr();
	}
};

std::string Stencila_version(void){
	return Stencila::version;
}

BOOST_PYTHON_MODULE(extension){
	// Declare converters
	to_python_converter<std::vector<std::string>, vector_to_list<std::string>>();

	// Declare exception translation and general Stencila functions
	def_Exception();
	def("version",Stencila_version);

	// Declare classes
	def_Component();
    def_Package();
    def_Stencil();
}
