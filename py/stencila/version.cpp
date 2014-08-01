#include <stencila/version.hpp>

#include <boost/python.hpp>

using namespace boost::python;

std::string Stencila_version(void){
    return Stencila::version;
}

void def_Version(void){
    def("version",Stencila_version);
}
