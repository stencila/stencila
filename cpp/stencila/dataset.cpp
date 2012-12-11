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
//! @brief Implmentations of Dataset methods which are unable to go into dataset.hpp because they return a Datatable

#include <stencila/dataset.hpp>
#include <stencila/datatable.hpp>

namespace Stencila {


Datatable Dataset::create(const std::string& name){
	return Datatable(name,this,false);
}

Datatable Dataset::table(const std::string& name){
	return Datatable(name,this);
}

Datatable Dataset::load(const std::string& name,const std::string& path){
	return Datatable(name,this).load(path);
}

Datatable Dataset::import(const std::string& name){
    //Check to see if this Datatable is already registered
    if(not value<int>("SELECT count(*) FROM stencila_datatables WHERE name==?",name)){
        execute("INSERT INTO stencila_datatables(name,source,status) VALUES(?,'table',2)",name);
    }
    return table(name);
}

Datatable Dataset::select(const std::string& sql, bool reuse, bool cache){
    std::string signature = boost::lexical_cast<std::string>(Hash(sql));
    std::string name = "stencila_"+signature;
    
    // Check whether signature is already in cache
    int exists = value<int>("SELECT count(*) FROM stencila_datatables WHERE signature==?",signature);
    // If not reusing cached table then drop existing table if it does exist
    if(!reuse and exists) {
        execute("DROP TABLE \""+name+"\"");
        execute("DELETE stencila_datatables WHERE signature==?",signature);
    }
    if(!reuse or !exists){
        //The extra "SELECT * FROM" in the following SQL is necessary for the correct interpretation of quoted
        //field names in the original SQL. If you don't treat the orginal SQL as a subquery then statements like
        // "SELECT "year",..." get treated as a string "year".
        execute("CREATE TEMPORARY TABLE \""+name+"\" AS SELECT * FROM ("+sql+ ")");
        if(cache) execute("INSERT INTO stencila_datatables(name,source,sql,signature,status) VALUES(?,'select',?,?,0)",name,sql,signature);
    }
    return table(name);
}

Datatable Dataset::clone(const std::string& original){
    std::string signature = boost::lexical_cast<std::string>(Hash());
    std::string name = "stencila_"+signature;
    
    execute("DROP TABLE IF EXISTS \""+name+"\"");
    execute("CREATE TEMPORARY TABLE \""+name+"\" AS SELECT * FROM \""+original+"\"");
    execute("INSERT INTO stencila_datatables(name,source,sql,signature,status) VALUES(?,'clone',?,?,0)",name,original,signature);
    
    return table(name);
}

}
