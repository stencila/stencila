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

//!	@file datacursor.hpp
//!	@brief Definition of class Datacursor

#pragma once

#include <sqlite3.h>

#include "datatypes.hpp"
#include "exception.hpp"

namespace Stencila {

class Datacursor {
	
private:
	
	sqlite3* db_;
	sqlite3_stmt* stmt_;
	bool executed_;
	bool more_;

public:
	
	Datacursor(sqlite3* db, std::string sql):
		db_(db),
		executed_(false),
		more_(false){
		if(sqlite3_prepare_v2(db_, sql.c_str(), -1, &stmt_, 0)!=SQLITE_OK){
			sqlite3_finalize(stmt_);
			throw Exception("sqlite3_prepare_v2(\""+sql+"\") failed : "+sqlite3_errmsg(db_));
		}
	}
	
	~Datacursor(void){
		if(sqlite3_finalize(stmt_)!=SQLITE_OK) throw Exception(std::string("sqlite3_finalize failed : ")+sqlite3_errmsg(db_));
	}
	
	bool more(void) const {
		return more_;
	}
	
	Datacursor& bind(unsigned int index,const std::string& value){
		if(sqlite3_bind_text(stmt_,index+1,value.c_str(),value.length(),SQLITE_STATIC)!=SQLITE_OK) {
			throw Exception("sqlite3_bind_text(\""+value+"\") failed : "+sqlite3_errmsg(db_));
		}
		return *this;
	}
	
	void next(void){
		int step = sqlite3_step(stmt_);
		if(step==SQLITE_ROW) more_ =  true;
		else if(step==SQLITE_DONE) more_ = false;
		else{
			throw Exception(std::string("sqlite3_step failed : ")+sqlite3_errmsg(db_));
		}
	}
	
	void execute(void){
		if(not executed_) {
			next();
			executed_ = true;
		}
	}
	
	void reset(void){
		if(sqlite3_clear_bindings(stmt_)!=SQLITE_OK)  Exception(std::string("sqlite3_clear_bindings failed : ")+sqlite3_errmsg(db_));
		if(sqlite3_reset(stmt_)!=SQLITE_OK)  Exception(std::string("sqlite3_reset failed : ")+sqlite3_errmsg(db_));
		executed_ = false;
	}
	
	unsigned int columns(void){
		execute();
		return sqlite3_column_count(stmt_);
	}
	
	std::string name(unsigned int column){
		execute();
		return sqlite3_column_name(stmt_,column);
	}
	
	std::vector<std::string> names(void) {
		std::vector<std::string> result;
		for(unsigned int i=0;i<columns();i++) result.push_back(name(i));
		return result;
	}
	
	const Datatype& type(unsigned int column){
		execute();
		switch(sqlite3_column_type(stmt_,column)){
			case SQLITE_NULL:
				return Null;
			break;
			case SQLITE_INTEGER:
				return Integer;
			break;
			case SQLITE_FLOAT:
				return Real;
			break;
			case SQLITE_TEXT:
				return Text;
			break;
		}
		throw Exception("Undefined column type");
	}
	
	std::vector<Datatype> types(void) {
		std::vector<Datatype> result;
		for(unsigned int i=0;i<columns();i++) result.push_back(type(i));
		return result;
	}
	
	template<typename Type>
	Type get(unsigned int column);
	
	template<typename Type = std::vector<std::string>>
	std::vector<Type> fetch(void) {
		std::vector<Type> rows;
		execute();
		while(more()) {
			Type row;
			for(unsigned int col=0;col<columns();col++) row.push_back(get<std::string>(col));
			rows.push_back(row);
			next();
		}
		return rows;
	}

	template<typename Type = std::string>
	Type value(void) {
		execute();
		if(more()) return get<Type>(0);
		else throw Exception("No rows selected");
	}
	
	template<typename Type = std::string>
	std::vector<Type> column(void) {
		std::vector<Type> column;
		execute();
		while(more()){
			column.push_back(get<Type>(0));
			next();
		}
		return column;
	}
	
	template<typename Type = std::vector<std::string>>
	Type row(void) {
		Type row;
		execute();
		if(more()){
			for(unsigned int col=0;col<columns();col++) row.push_back(get<std::string>(col));
		}
		return row;
	}
	
};

template<>
inline
int Datacursor::get<int>(unsigned int column){
	return sqlite3_column_int(stmt_, column);
}

template<>
inline
double Datacursor::get<double>(unsigned int column){
	return sqlite3_column_double(stmt_, column);
}

template<>
inline
std::string Datacursor::get<std::string>(unsigned int column){
	return reinterpret_cast<const char *>(sqlite3_column_text(stmt_, column));
}

}