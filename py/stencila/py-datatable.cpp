#include <stencila/datatable.hpp>
using namespace Stencila;

#include "py-extension.hpp"

/*
object select_tuple(tuple args, dict kwargs){
    //!@todo Need to test if this is a slice or not
    Datatable& self = extract<Datatable&>(args[0]);
    tuple rowsCols = extract<tuple>(args[1]);
    
    object rows = rowsCols[0];
    extract<unsigned int> rows_is_int(rows);
    if(rows_is_int.check()){
        //int row = rows_is_int();
    } else {
        extract<slice> rows_is_slice(rows);
    }
    
    return object(self.value<>(0,0));
}
*/

void Datatable_define(void){
    class_<Datatable,bases<>>("DatatableBase")
        .def(init<std::string>())
        .def(init<std::string,Dataset*>())
    
        .def("rows", &Datatable::rows) 
        .def("columns", &Datatable::columns)
        .def("names", &Datatable::names)
        .def("indices", &Datatable::indices)
    
        .def("add",
            static_cast<Datatable& (Datatable::*)(const std::string& column, const Datatype& type)>(&Datatable::add),
            return_value_policy<reference_existing_object>()
        )

        //.def("__getitem__",raw_function(select_tuple))
    ;
}
