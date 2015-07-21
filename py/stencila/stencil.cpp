#include <string>
#include <memory>

#include <stencila/stencil.hpp>

#include "context.hpp"

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Stencil_html_get_overloads,html,0,2)

Stencil& Stencil_render(Stencil& self, object context){
    // Use supplied Python Context to create a C++ side PythonContext
    auto python_context = std::make_shared<PythonContext>(context);
    // Render within this context
    self.render(python_context);
    return self;
}

void def_Stencil(void){
    class_<Stencil,bases<Component>>("Stencil")
        .def(init<std::string>())

        .def("html",
            static_cast<std::string (Stencil::*)(bool,bool) const>(&Stencil::html),
            Stencil_html_get_overloads()
        )
        .def("html",
            static_cast<Stencil& (Stencil::*)(const std::string&)>(&Stencil::html),
            return_self<>()
        )

        .def("title",&Stencil::title)
        .def("description",&Stencil::description)
        .def("keywords",&Stencil::keywords)
        .def("authors",&Stencil::authors)

        .def("render",Stencil_render,return_self<>())

        .def("serve",&Stencil::serve,return_self<>())
        .def("view",&Stencil::view,return_self<>())

        .def("compile",&Stencil::compile,return_self<>())
    ;
}