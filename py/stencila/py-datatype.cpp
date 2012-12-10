#include <stencila/datatypes.hpp>
using namespace Stencila;

#include "py-extension.hpp"

void Datatype_define(void){
    class_<Datatype,bases<>>("Datatype")
        .def(init<char>())
    ;
}
