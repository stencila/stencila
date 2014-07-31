#include <string>

#include <stencila/stencil.hpp>
#include <stencila/python-context.hpp>

#include "extension.hpp"

using namespace Stencila;
using namespace boost::python;

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Stencil_content_set_overloads,content,2,2)

Stencil& Stencil_render(Stencil& self, object python_context){
    // Use supplied Python Context to create a C++ side PythonContext
    PythonContext context(python_context);
    // Render within this context
    self.render(context);
    return self;
}

void def_Stencil(void){
    class_<Stencil,bases<Component>>("Stencil")

        .def("content",
            static_cast<std::string (Stencil::*)(const std::string&) const>(&Stencil::content)
        )
        .def("content",
            static_cast<Stencil& (Stencil::*)(const std::string&,const std::string&)>(&Stencil::content),
            Stencil_content_set_overloads(
                (arg("format"),arg("content")),
                "Set the stencil's content"
            )[return_self<>()]
        )

        .def("html",
            static_cast<std::string (Stencil::*)(void) const>(&Stencil::html)
        )
        .def("html",
            static_cast<Stencil& (Stencil::*)(const std::string&)>(&Stencil::html),
            return_self<>()
        )

        .def("render",
            Stencil_render,
            return_self<>()
        )
    ;
}