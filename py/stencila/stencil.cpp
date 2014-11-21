#include <string>

#include <stencila/stencil.hpp>

#include "context.hpp"

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Stencil_html_set_overloads,html,0,2)

Stencil& Stencil_render(Stencil& self, object python_context){
    // Use supplied Python Context to create a C++ side PythonContext
    PythonContext context(python_context);
    // Render within this context
    self.render(&context);
    return self;
}

void def_Stencil(void){
    class_<Stencil,bases<Component>>("Stencil")
        .def(init<std::string>())

        .def("html",
            static_cast<std::string (Stencil::*)(bool,bool) const>(&Stencil::html),
            Stencil_html_set_overloads()
        )
        .def("html",
            static_cast<Stencil& (Stencil::*)(const std::string&)>(&Stencil::html),
            return_self<>()
        )

        .def("render",
            Stencil_render,
            return_self<>()
        )

        .def("page",
            &Stencil::page
        )
    ;
}