#include <string>

#include <stencila/stencil.hpp>

#include "context.hpp"

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Stencil_html_set_overloads,html,0,2)

Stencil& Stencil_render(Stencil& self, object context){
    //! @todo Garbage collection of PythonContext is not correcly handles here
    // Use supplied Python Context to create a C++ side PythonContext
    PythonContext* python_context = new PythonContext(context);
    // Render within this context
    self.render(python_context);
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

        .def("title",&Stencil::title)
        .def("description",&Stencil::description)
        .def("keywords",&Stencil::keywords)
        .def("authors",&Stencil::authors)

        .def("render",Stencil_render,return_self<>())

        .def("compile",&Stencil::compile,return_self<>())
    ;
}