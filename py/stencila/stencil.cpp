#include <string>
#include <memory>

#include <stencila/stencil.hpp>

#include "context.hpp"

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Stencil_html_get_overloads,html,0,2)


Stencil& Stencil_attach(Stencil& self, object context){
    self.attach(std::make_shared<PythonContext>(context));
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

        .def("cila",
            static_cast<std::string (Stencil::*)(void) const>(&Stencil::cila)
        )
        .def("cila",
            static_cast<Stencil& (Stencil::*)(const std::string&)>(&Stencil::cila),
            return_self<>()
        )

        .def("source",
            static_cast<std::string (Stencil::*)(void) const>(&Stencil::source)
        )
        .def("source",
            static_cast<Stencil& (Stencil::*)(const std::string&)>(&Stencil::source),
            return_self<>()
        )

        .def("read",&Stencil::read,return_self<>())
        .def("write",&Stencil::write,return_self<>())

        .def("title",&Stencil::title)
        .def("description",&Stencil::description)
        .def("keywords",&Stencil::keywords)
        .def("authors",&Stencil::authors)

        .def("attach",Stencil_attach,return_self<>())
        .def("detach",&Stencil::detach,return_self<>())
        .def("render",
            static_cast<Stencil& (Stencil::*)(void)>(&Stencil::render),
            return_self<>()
        )

        .def("serve",&Stencil::serve,return_self<>())
        .def("view",&Stencil::view,return_self<>())

        .def("page",
            static_cast<std::string (Stencil::*)(void) const>(&Stencil::page)
        )
        .def("page",
            static_cast<Stencil& (Stencil::*)(const std::string&)>(&Stencil::page),
            return_self<>()
        )
    ;
}