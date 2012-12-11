#include <stencila/dataset.hpp>
#include <stencila/datatable.hpp>
using namespace Stencila;

#include "py-extension.hpp"

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(
    save_overloads,
    Dataset::save, 
    0, 1
)    
    
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(
    indices_overloads,
    Dataset::indices, 
    0, 1
)

object Datacursor_get(Datacursor& cursor, unsigned int column){
    const char type = cursor.type(column).code;
    switch(type){
        case 'n': return object();
        case 'i': return object(cursor.get<int>(column));
        case 'r': return object(cursor.get<double>(column));
        case 't': return object(cursor.get<std::string>(column));
    }
    return object();
}

object Dataset_fetch(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list rows;
    while(cursor.more()){
        list row;
        for(unsigned int column=0;column<cursor.columns();column++){
            row.append(Datacursor_get(cursor,column));
        }
        rows.append(row);
        cursor.next();
    }
    return rows;
}

object Dataset_value(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    if(cursor.more()) return Datacursor_get(cursor,0);
    else throw Exception("No rows returned");
}

object Dataset_column(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor  cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list column;
    while(cursor.more()){
        column.append(Datacursor_get(cursor,0));
        cursor.next();
    }
    return column;
}

object Dataset_row(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor cursor= self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list row;
    if(cursor.more()){
        for(unsigned int column=0;column<cursor.columns();column++){
            row.append(Datacursor_get(cursor,column));
        }
        cursor.next();
    }
    return row;
}

void Dataset_define(void){
    
    class_<Dataset,bases<>>("Dataset")
        .def(init<std::string>())

        .def("save",&Dataset::save,
            save_overloads(
                args("path","backup"),
                "Save the dataset to path"
            )[return_value_policy<reference_existing_object>()]
        )

        .def("tables", &Dataset::tables)
        .def("indices",
            &Dataset::indices,
            indices_overloads(args("table"))
        )

        .def("execute",
            static_cast<Dataset& (Dataset::*)(const std::string& sql)>(&Dataset::execute),
            return_value_policy<reference_existing_object>()
        )

        .def("fetch", raw_function(Dataset_fetch))
        .def("value", raw_function(Dataset_value))
        .def("column",  raw_function(Dataset_column))
        .def("row",raw_function(Dataset_row))

        .def("table", &Dataset::table)
    ;
}
