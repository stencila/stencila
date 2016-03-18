#include <boost/python.hpp>
using namespace boost::python;

#include <stencila/sheet.hpp>
using namespace Stencila;

#include "spread.hpp"

Sheet& Sheet_attach(Sheet& self, object context) {
    self.attach(std::make_shared<PythonSpread>(context));
    return self;
}

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

        .def("page",
            static_cast<std::string (Sheet::*)(void) const>(&Sheet::page)
        )
        .def("page",
            static_cast<Sheet& (Sheet::*)(const std::string&)>(&Sheet::page),
            return_self<>()
        )

        .def("attach", Sheet_attach, return_self<>())
        .def("detach", &Sheet::detach, return_self<>())

        .def("update", static_cast<Sheet& (Sheet::*)(void)>(&Sheet::update), return_self<>())
    ;
}
