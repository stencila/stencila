/*
Copyright (c) 2012, Nokome Bentley, nokome.bentley@stenci.la

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#pragma once

#include <string>
#include <vector>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

#include <sqlite3.h>

#include "datacursor.hpp"
#include "exception.hpp"

namespace Stencila {
	
class Datatable;
	
class Dataset {
	
private:
	std::string uri_;
	sqlite3* db_;

public:
	
	Dataset(std::string uri = ""):
		uri_(uri){
		
		std::string path;
		if(uri.length()==0) path = ":memory:";
		else path = uri_;
			
		int code_open = sqlite3_open(path.c_str(), &db_);
		if(code_open) throw Exception(("Unable to open : "+uri_).c_str());
	}
	
	~Dataset(void){
		if(db_) sqlite3_close(db_);
	}
	
	//////////////////////////////////////////
	
	void save(std::string uri){
			
		sqlite3* to;
		int code_open = sqlite3_open(uri.c_str(), &to);
		if(code_open) throw Exception(("Unable to open : "+uri).c_str());
		
		/* See http://www.sqlite.org/backup.html */
		sqlite3_backup* backup = sqlite3_backup_init(to, "main", db_, "main");
		if(backup){
			sqlite3_backup_step(backup,-1);
			sqlite3_backup_finish(backup);
		}
		
		uri_ = uri;
	}
	
	//////////////////////////////////////////
	
	std::vector<std::string> tables(void) {
		return column("SELECT name FROM sqlite_master WHERE type=='table';");
	}
	
	std::vector<std::string> indices(std::string table="") {
		std::string sql = "SELECT name FROM sqlite_master WHERE type=='index' ";
		if(table.length()==0) return column(sql);
		else  return column(sql+"AND tbl_name=='"+table+"'");
	}
	
	///////////////////////////////////////////
	
	void execute(std::string sql) {
		int exec = sqlite3_exec(db_,sql.c_str(),0,0,0);
		if(exec!=SQLITE_OK) throw Exception(sqlite3_errmsg(db_));
	}
	
	Datacursor query(std::string sql){
		return Datacursor(db_,sql);
	}
	
	template<typename Type = std::vector<std::string>>
	std::vector<Type> fetch(std::string sql) {
		return Datacursor(db_,sql).fetch<Type>();
	}

	template<typename Type = std::string>
	Type value(std::string sql) {
		return Datacursor(db_,sql).value<Type>();
	}
	
	template<typename Type = std::string>
	std::vector<Type> column(std::string sql) {
		return Datacursor(db_,sql).column<Type>();
	}
	
	template<typename Type = std::vector<std::string>>
	Type row(std::string sql) {
		return Datacursor(db_,sql).row<Type>();
	}
	
	///////////////////////////////////////////////////////
	
	Datatable table(std::string name);
};
	
} 