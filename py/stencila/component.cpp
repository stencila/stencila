#include <vector>
#include <string>

#include <stencila/component.hpp>

#include <boost/python.hpp>

using namespace Stencila;
using namespace boost::python;

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_path_set_overloads,path,1,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_path_get_overloads,path,1,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_destroy_overloads,destroy,0,0)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_create_overloads,create,1,2)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_delete_file_overloads,delete_file,1,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_read_overloads,read,0,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_write_overloads,write,0,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Component_commit_overloads,commit,0,1)


Component* Component_instantiate(const std::string& type, const std::string& content, const std::string& format) {
    // Because this may be called from another thread (e.g. the server thread)
    // it is necessary to obtain the Python GIL before calling Python code
    PyGILState_STATE py_gil_state = PyGILState_Ensure();
    auto instantiate_function = import("stencila").attr("instantiate");
    auto component_object = instantiate_function(type, content, format);
    Component* component = extract<Component*>(component_object);
    PyGILState_Release(py_gil_state);
    return component;
}


std::vector<std::string> Component_grab(const std::string& address) {
    Component::Instance instance = Component::get(address);
    Component* component = instance.pointer();
    return {
        component->address(),
        component->path(),
        instance.type_name()
    };
}


void def_Component(void){
    class_<Component,bases<>>("Component")

        .def("address",
            static_cast<std::string (Component::*)(void) const>(&Component::address)
        )

        .def("path",
            static_cast<std::string (Component::*)(bool) const>(&Component::path),
            Component_path_get_overloads(
                arg("ensure")=false,
                "Get the component's working directory"
            )
        )
        .def("path",
            static_cast<Component& (Component::*)(const std::string&)>(&Component::path),
            Component_path_set_overloads(
                arg("path"),
                "Set the component's working directory"
            )[return_self<>()]
        )

        .def("delete_file",
            static_cast<Component& (Component::*)(const std::string&)>(&Component::delete_file),
            Component_delete_file_overloads(
                arg("path"),
                "Delete a file in the component's working directory"
            )[return_self<>()]
        )

        .def("read",
            static_cast<Component& (Component::*)(const std::string&)>(&Component::read),
            Component_read_overloads(
                arg("path")="",
                "Read the component from a filesystem path"
            )[return_self<>()]
        )

        .def("write",
            static_cast<Component& (Component::*)(const std::string&)>(&Component::write),
            Component_write_overloads(
                arg("path")="",
                "Write the component to a filesystem path"
            )[return_self<>()]
        )

        .def("destroy",
            static_cast<Component& (Component::*)(void)>(&Component::destroy),
            Component_destroy_overloads(
                "Destroy the component's working directory"
            )[return_self<>()]
        )

        .def("commit",
            &Component::commit,
            Component_commit_overloads(
                arg("message")="",
                "Commit the component"
            )[return_self<>()]
        )
    ;

    def("grab", Component_grab);
}
