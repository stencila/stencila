#include <stencila/package.hpp>

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

void def_Package(void){
    class_<Package,bases<Component>>("Package");
}
