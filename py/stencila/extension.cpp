#include "extension.hpp"

BOOST_PYTHON_MODULE(extension){
	using namespace bp;
	def("version",Stencila_version);
}
