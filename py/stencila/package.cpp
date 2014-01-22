#include <string>

#include <stencila/package.hpp>
using namespace Stencila;

#include "extension.hpp"

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Package_create_overloads,create,1,2)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Package_destroy_path_overloads,destroy,1,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Package_destroy_all_overloads,destroy,0,0)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Package_read_overloads,read,0,1)
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(Package_write_overloads,write,0,1)

void def_Package(void){
    class_<Package,bases<>>("Package")

        .def("title",static_cast<const std::string& (Package::*)(void) const>(&Package::title),return_value_policy<copy_const_reference>())
        .def("title",static_cast<Package& (Package::*)(const std::string&)>(&Package::title),return_self<>())

        .def("description",static_cast<const std::string& (Package::*)(void) const>(&Package::description),return_value_policy<copy_const_reference>())
        .def("description",static_cast<Package& (Package::*)(const std::string&)>(&Package::description),return_self<>())

        .def("keywords",static_cast<const std::vector<std::string>& (Package::*)(void) const>(&Package::keywords),return_value_policy<copy_const_reference>())
        .def("keywords",static_cast<Package& (Package::*)(const std::vector<std::string>&)>(&Package::keywords),return_self<>())

        .def("authors",static_cast<const std::vector<std::string>& (Package::*)(void) const>(&Package::authors),return_value_policy<copy_const_reference>())
        .def("authors",static_cast<Package& (Package::*)(const std::vector<std::string>&)>(&Package::authors),return_self<>())

        .def("path",static_cast<const std::string& (Package::*)(void) const>(&Package::path),return_value_policy<copy_const_reference>())
        .def("path",static_cast<Package& (Package::*)(const std::string&)>(&Package::path),return_self<>())

        .def("create",static_cast<Package& (Package::*)(const std::string&,const std::string&)>(&Package::create),Package_create_overloads(
            (arg("path"),arg("content")),
            "Create a file in the package's working directory"
        )[return_self<>()])
        
        .def("destroy",static_cast<Package& (Package::*)(const std::string&)>(&Package::destroy),Package_destroy_path_overloads(
            arg("path"),
            "Detroy a file in the package's working directory"
        )[return_self<>()])

        .def("destroy",static_cast<Package& (Package::*)(void)>(&Package::destroy),Package_destroy_all_overloads(
            "Destroy the package's working directory"
        )[return_self<>()])

        .def("read",static_cast<Package& (Package::*)(const std::string&)>(&Package::read),Package_read_overloads(
            arg("path")="",
            "Read the package"
        )[return_self<>()])

        .def("write",static_cast<Package& (Package::*)(const std::string&)>(&Package::write),Package_write_overloads(
            arg("path")="",
            "Write the package"
        )[return_self<>()])

        .def("commit",&Package::commit,return_self<>())
    ;
}