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

//!	@file dataset.cpp
//! @brief Definition of Dataset methods which are unable to go into dataset.hpp

#include "dataset.hpp"
#include "datatable.hpp"
#include "hashing.hpp"

namespace Stencila {

inline Datatable Dataset::table(const std::string& name){
	return Datatable(name,this);
}

inline Datatable Dataset::import(const std::string& name){
	//Check to see if this Datatable is already registered
	if(not value<int>("SELECT count(*) FROM stencila_datatables WHERE name==?",name)){
		execute("INSERT INTO stencila_datatables(name,source,status) VALUES(?,'table',2)",name);
	}
	return Datatable(name,this);
}

inline Datatable Dataset::select(const std::string& sql){
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