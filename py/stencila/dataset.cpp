#pragma once

#include <stencila/dataset.hpp>
#include <stencila/dataset.cpp>

namespace Stencila {
namespace Python {
namespace DatasetBindings {
    
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
    
typedef Dataset& (Dataset::* execute_method)(const std::string& sql);
    
object value_to_object(Datacursor& cursor, unsigned int column){
    const char type = cursor.type(column).code;
    switch(type){
        case 'n': return object();
        case 'i': return object(cursor.get<int>(column));
        case 'r': return object(cursor.get<double>(column));
        case 't': return object(cursor.get<std::string>(column));
    }
    return object();
}

object fetch(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list rows;
    while(cursor.more()){
        list row;
        for(unsigned int column=0;column<cursor.columns();column++){
            row.append(value_to_object(cursor,column));
        }
        rows.append(row);
        cursor.next();
    }
    return rows;
}

object value(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    if(cursor.more()) return value_to_object(cursor,0);
    else throw Exception("No rows returned");
}

object column(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor  cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list column;
    while(cursor.more()){
        column.append(value_to_object(cursor,0));
        cursor.next();
    }
    return column;
}

object row(tuple args, dict kwargs){
    Dataset& self = extract<Dataset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Datacursor cursor= self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list row;
    if(cursor.more()){
        for(unsigned int column=0;column<cursor.columns();column++){
            row.append(value_to_object(cursor,column));
        }
        cursor.next();
    }
    return row;
}

void bind(void){
    
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

        .def("fetch", raw_function(fetch))
        .def("value", raw_function(value))
        .def("column",  raw_function(column))
        .def("row",raw_function(row))

        .def("table", &Dataset::table)
    ;
}

}}}


