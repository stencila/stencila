/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file dataset.cpp
//! @brief Implmentations of Dataset methods which are unable to go into dataset.hpp

#include <stencila/dataset.hpp>
#include <stencila/datatable.hpp>
#include <stencila/hashing.hpp>

namespace Stencila {

inline std::string Dataset_create_helper(const std::string& column, const Datatype& type){
	return column + " " + type.sql();
}

template<typename... Columns>
inline std::string Dataset_create_helper(const std::string& column, const Datatype& type, Columns... columns){
	return Dataset_create_helper(column,type) + "," + Dataset_create_helper(columns...);
}

template<typename... Columns>
Datatable Dataset::create(const std::string& name, Columns... columns){
	std::string sql = "CREATE TABLE " + name + "(" + Dataset_create_helper(columns...) + ");";
	execute(sql);
	return Datatable(name,this);
}
	
Datatable Dataset::table(const std::string& name){
	return Datatable(name,this);
}

Datatable Dataset::import(const std::string& name){
	//Check to see if this Datatable is already registered
	if(not value<int>("SELECT count(*) FROM stencila_datatables WHERE name==?",name)){
		execute("INSERT INTO stencila_datatables(name,source,status) VALUES(?,'table',2)",name);
	}
	return Datatable(name,this);
}

Datatable Dataset::load(const std::string& name,const std::string& path){
	return Datatable(name,this).load(path);
}

inline std::string Dataset_index_helper(const std::string& column){
	return column;
}

template<typename... Columns>
inline std::string Dataset_index_helper(const std::string& separator, const std::string& column, Columns... columns){
	return Dataset_index_helper(column) + separator + Dataset_index_helper(separator,columns...);
}

template<typename... Columns>
void Dataset::index(const std::string& table, Columns... columns){
	std::string sql = "CREATE INDEX " + table + Dataset_index_helper("_",columns...) + "_index ON " + table + "(" + Dataset_index_helper(",",columns...) + ");";
	execute(sql);
}

Datatable Dataset::select(const std::string& sql){
	std::string signature = boost::lexical_cast<std::string>(Hash(sql));
	std::string name = "stencila_"+signature;
	
	// Check whether id is already in cache and only execute SQL if it is not
	int exists = value<int>("SELECT count(*) FROM stencila_datatables WHERE signature==?",signature);
	if(!exists){
		execute("CREATE TEMPORARY TABLE \""+name+"\" AS "+sql);
		execute("INSERT INTO stencila_datatables(name,source,sql,signature,status) VALUES(?,'select',?,?,0)",name,sql,signature);
	}
	
	return table(name);
}

}