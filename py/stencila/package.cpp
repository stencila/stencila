#include <stencila/package.hpp>
using namespace Stencila;

#include "extension.hpp"

void def_Package(void){
    class_<Package,bases<Component>>("Package");
}
