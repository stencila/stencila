#include <stencila/package.hpp>

#include "extension.hpp"

void def_Package(void){
    class_<Package,bases<Component>>("Package");
}
