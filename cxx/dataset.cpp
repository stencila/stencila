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
//! 	@brief Definition of Dataset methods which are unable to go into dataset.hpp

#include "dataset.hpp"
#include "datatable.hpp"
#include "hashing.hpp"

namespace Stencila {

inline Datatable Dataset::table(const std::string& name){
	return Datatable(name,this);
}

inline Datatable Dataset::select(const std::string& sql){
	std::string id = boost::lexical_cast<std::string>(Hash(sql));
	std::string name = "stencila_"+id;
	
	// Check whether id is already in cache and only execute SQL if it is not
	int exists = value<int>("SELECT count(*) FROM stencila_cache WHERE id=="+id);
	if(!exists){
		execute("CREATE TEMPORARY TABLE \""+name+"\" AS "+sql);
		
		Datacursor insert = cursor("INSERT INTO stencila_cache(id,name,status,sql) VALUES(?,?,0,?)");
		insert.bind(0,id);
		insert.bind(1,name);
		insert.bind(2,sql);
		insert.execute();
	}
	
	return table(name);
}

}