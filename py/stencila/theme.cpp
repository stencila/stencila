#include <stencila/theme.hpp>

#include "extension.hpp"
#include "context.hpp"

void def_Theme(void){
    class_<Theme,bases<Component>>("Theme")
        .def(init<std::string>())

        .def("title",&Theme::title)
        .def("description",&Theme::description)
        .def("keywords",&Theme::keywords)
        .def("authors",&Theme::authors)

        .def("serve",&Theme::serve,return_self<>())
        .def("view",&Theme::view,return_self<>())

        .def("compile",&Theme::compile,return_self<>())
    ;
}