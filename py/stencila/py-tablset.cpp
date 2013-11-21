#include <stencila/tables/tableset.hpp>
#include <stencila/tables/table.hpp>
using namespace Stencila;
using namespace Stencila::Tables;

#include "py-extension.hpp"

BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(
    save_overloads,
    Tableset::save, 
    0, 1
)    
    
BOOST_PYTHON_MEMBER_FUNCTION_OVERLOADS(
    indices_overloads,
    Tableset::indices, 
    0, 1
)

object Cursor_get(Cursor& cursor, unsigned int column){
    const char type = cursor.type(column).code;
    switch(type){
        case 'n': return object();
        case 'i': return object(cursor.get<int>(column));
        case 'r': return object(cursor.get<double>(column));
        case 't': return object(cursor.get<std::string>(column));
    }
    return object();
}

object Tableset_fetch(tuple args, dict kwargs){
    Tableset& self = extract<Tableset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Cursor cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list rows;
    while(cursor.more()){
        list row;
        for(unsigned int column=0;column<cursor.columns();column++){
            row.append(Cursor_get(cursor,column));
        }
        rows.append(row);
        cursor.next();
    }
    return rows;
}

object Tableset_value(tuple args, dict kwargs){
    Tableset& self = extract<Tableset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Cursor cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    if(cursor.more()) return Cursor_get(cursor,0);
    else throw Exception("No rows returned");
}

object Tableset_column(tuple args, dict kwargs){
    Tableset& self = extract<Tableset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Cursor  cursor = self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list column;
    while(cursor.more()){
        column.append(Cursor_get(cursor,0));
        cursor.next();
    }
    return column;
}

object Tableset_row(tuple args, dict kwargs){
    Tableset& self = extract<Tableset&>(args[0]);
    std::string sql = extract<std::string>(args[1]);
    
    Cursor cursor= self.cursor(sql);
    cursor.prepare();
    cursor.begin();
    
    list row;
    if(cursor.more()){
        for(unsigned int column=0;column<cursor.columns();column++){
            row.append(Cursor_get(cursor,column));
        }
        cursor.next();
    }
    return row;
}

void Tableset_define(void){
    
    class_<Tableset,bases<>>("Tableset")
        .def(init<std::string>())

        .def("save",&Tableset::save,
            save_overloads(
                args("path","backup"),
                "Save the dataset to path"
            )[return_value_policy<reference_existing_object>()]
        )

        .def("tables", &Tableset::tables)
        .def("indices",
            &Tableset::indices,
            indices_overloads(args("table"))
        )

        .def("execute",
            static_cast<Tableset& (Tableset::*)(const std::string& sql)>(&Tableset::execute),
            return_value_policy<reference_existing_object>()
        )

        .def("fetch", raw_function(Tableset_fetch))
        .def("value", raw_function(Tableset_value))
        .def("column",  raw_function(Tableset_column))
        .def("row",raw_function(Tableset_row))

        .def("table", &Tableset::table)
    ;
}
