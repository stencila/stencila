#include <stencila/tables/table.hpp>
using namespace Stencila;
using namespace Stencila::Tables;

#include "py-extension.hpp"

/*
object select_tuple(tuple args, dict kwargs){
    //!@todo Need to test if this is a slice or not
    Table& self = extract<Table&>(args[0]);
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

void Table_define(void){
    class_<Table,bases<>>("TableBase")
        .def(init<std::string>())
        .def(init<std::string,Tableset*>())
    
        .def("rows", &Table::rows) 
        .def("columns", &Table::columns)
        .def("labels", &Table::labels)
        .def("indices", &Table::indices)
    
        .def("add",
            static_cast<Table& (Table::*)(const std::string& column, const Datatype& type)>(&Table::add),
            return_value_policy<reference_existing_object>()
        )

        //.def("__getitem__",raw_function(select_tuple))
    ;
}
