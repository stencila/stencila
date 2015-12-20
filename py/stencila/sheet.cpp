#include <boost/python.hpp>
using namespace boost::python;

#include <stencila/sheet.hpp>
using namespace Stencila;

void def_Sheet(void){
    class_<Sheet,bases<Component>>("Sheet")
        .def(init<std::string>())

        .def("read",&Sheet::read,return_self<>())
        .def("write",&Sheet::write,return_self<>())

        .def("title",&Sheet::title)
        .def("description",&Sheet::description)
        .def("keywords",&Sheet::keywords)
        .def("authors",&Sheet::authors)

        .def("serve",&Sheet::serve,return_self<>())
        .def("view",&Sheet::view,return_self<>())

        .def("compile",&Sheet::compile,return_self<>())
    ;
}
