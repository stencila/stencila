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

#include "sqlite.hpp"
#include "datatypes.hpp"
#include "exception.hpp"

namespace Stencila {

class Datacursor {
	
private:
	
	sqlite3* db_;
	std::string sql_;
	sqlite3_stmt* stmt_;
	bool begun_;
	bool more_;

public:
	
	Datacursor(sqlite3* db, const std::string& sql):
		db_(db),
		sql_(sql),
		stmt_(0),
		begun_(false),
		more_(false){
	}
	
	template<typename... Parameters>
	Datacursor(sqlite3* db, const std::string& sql, Parameters&... pars):
		db_(db),
		sql_(sql),
		stmt_(0),
		begun_(false),
		more_(false){
		prepare();
		use(pars...);
	}
	
	~Datacursor(void){
		if(stmt_){
			STENCILA_SQLITE_TRY(db_,sqlite3_finalize(stmt_));
		}
	}
	
	const std::string& sql(void) const {
		return sql_;
	}
	
	bool more(void) const {
		return more_;
	}
	
	Datacursor& prepare(void){
		STENCILA_SQLITE_TRY(db_,sqlite3_prepare_v2(db_, sql_.c_str(), -1, &stmt_, 0));
		return *this;
	}
	
	//! @name Parameter binding methods
	//! @brief Bind values to parameters in SQL
	//! @{
	//! @warning Calls to Datacursor::bind methods must be preceded by a call to Datacursor::prepare
		
	Datacursor& bind(unsigned int index){
		STENCILA_SQLITE_TRY(db_,sqlite3_bind_null(stmt_,index));
		return *this;
	}
	
	Datacursor& bind(unsigned int index,const int& value){
		STENCILA_SQLITE_TRY(db_,sqlite3_bind_int(stmt_,index,value));
		return *this;
	}
	
	Datacursor& bind(unsigned int index,const double& value){
		STENCILA_SQLITE_TRY(db_,sqlite3_bind_double(stmt_,index,value));
		return *this;
	}
	
	Datacursor& bind(unsigned int index,const std::string& value){
		STENCILA_SQLITE_TRY(db_,sqlite3_bind_text(stmt_,index,value.c_str(),value.length(),SQLITE_STATIC));
		return *this;
	}
	
	template<
		typename Parameter,
		typename... Parameters
	>
	Datacursor& use(const Parameter& par, const Parameters&... pars){
		int count = sqlite3_bind_parameter_count(stmt_);
		int index = count - sizeof...(Parameters);
		bind(index,par);
		use(pars...);
		return *this;
	}
	
	Datacursor& use(void){
		return *this;
	}
	
	//! @}
	
	void reset(void){
		STENCILA_SQLITE_TRY(db_,sqlite3_clear_bindings(stmt_));
		STENCILA_SQLITE_TRY(db_,sqlite3_reset(stmt_));
		begun_ = false;
	}
	
	void begin(void){
		if(not begun_) {
			prepare();
			next();
			begun_ = true;
		}
	}
	
	void execute(void){
		//If a statement has already been prepared then sqlite3_step that...
		//sqlite3_step does not always return SQLITE_OK on success so do not wrap it in STENCILA_SQLITE_TRY
		if(stmt_){
			sqlite3_step(stmt_);
		}
		//Otherwise use the sqlite3_exec shortcut function to prepare, step and finalise in one
		else {
			STENCILA_SQLITE_TRY(db_,sqlite3_exec(db_,sql_.c_str(),0,0,0));
		}
	}
	
	template<typename... Parameters>
	void execute(const Parameters... pars){
		prepare();
		use(pars...);
		execute();
	}
	
	//! @warning Must be preceded by a call to Datacursor::prepare
	void next(void){
		int code = sqlite3_step(stmt_);
		if(code==SQLITE_ROW) {
			// sqlite3_step() has another row ready
			more_ =  true;
		}
		else if(code==SQLITE_DONE) {
			// sqlite3_step() has finished executing
			more_ = false; 
		}
		else{
			STENCILA_SQLITE_THROW(db_,code);
		}
	}
	
	unsigned int columns(void){
		begin();
		return sqlite3_column_count(stmt_);
	}
	
	std::string name(unsigned int column){
		begin();
		return sqlite3_column_name(stmt_,column);
	}
	
	std::vector<std::string> names(void) {
		std::vector<std::string> result;
		for(unsigned int i=0;i<columns();i++) result.push_back(name(i));
		return result;
	}
	
	const Datatype& type(unsigned int column){
		begin();
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
	
	template<
		typename Type = std::vector<std::string>,
		typename... Parameters
	>
	std::vector<Type> fetch(const Parameters&... pars) {
		std::vector<Type> rows;
		prepare();
		use(pars...);
		begin();
		while(more()) {
			Type row;
			for(unsigned int col=0;col<columns();col++) row.push_back(get<std::string>(col));
			rows.push_back(row);
			next();
		}
		return rows;
	}

	template<
		typename Type = std::string,
		typename... Parameters
	>
	Type value(const Parameters&... pars) {
		prepare();
		use(pars...);
		begin();
		if(more()) return get<Type>(0);
		else throw Exception("No rows selected");
	}
	
	template<
		typename Type = std::string,
		typename... Parameters
	>
	std::vector<Type> column(const Parameters&... pars) {
		std::vector<Type> column;
		prepare();
		use(pars...);
		begin();
		while(more()){
			column.push_back(get<Type>(0));
			next();
		}
		return column;
	}
	
	template<
		typename Type = std::vector<std::string>,
		typename... Parameters
	>
	Type row(const Parameters&... pars) {
		Type row;
		prepare();
		use(pars...);
		begin();
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
float Datacursor::get<float>(unsigned int column){
	return sqlite3_column_double(stmt_, column);
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