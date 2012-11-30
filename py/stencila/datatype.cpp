#include <stencila/datatypes.hpp>

#include "extension.hpp"

using namespace Stencila;

void Datatype_define(void){
    class_<Datatype,bases<>>("Datatype")
        .def(init<char>())
    ;
}
