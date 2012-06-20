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

#include "exception.hpp"
#include "dataset.hpp"

namespace Stencila {
	
class Datatable {
	
private:
	
	Dataset* dataset_;
	std::string name_;

public:
	
	//!	@name Constructors
	//!	@{

	Datatable(Dataset* dataset, std::string name):
		dataset_(dataset),
		name_(name){
	}
	
	//!	@}
	
	
	//!	@name Getter and setter methods
	//!	@{
	
	Dataset& dataset(void) {
		return *dataset_;
	}
	
	std::string name(void) {
		return name_;
	}
	
	//! 	@}
	
	
	//!	@name Attribute methods
	//! 	@{
	
	//! 	@desc The number of rows
	//!	@param sql The SQL to execute
	//!	@return number
	unsigned int rows(void) {
		return dataset().value<int>("SELECT count(*) FROM "+name());
	}
	
	unsigned int columns(void) {
		return dataset().query("SELECT * FROM "+name()).columns();
	}
	
	std::vector<unsigned int> dimensions(void) {
		return {rows(),columns()};
	}
	
	//!	@desc The name of a column
	//!	@param column The column index
	//!	@return Name of column
	std::string name(unsigned int column) {
		return dataset().query("SELECT * FROM "+name()).name(column);
	}
	
	std::vector<std::string> names(void) {
		return dataset().query("SELECT * FROM "+name()).names();
	}
	
	Datatype type(unsigned int column) {
		return dataset().query("SELECT * FROM "+name()).type(column);
	}
	
	std::vector<Datatype> types(void) {
		return dataset().query("SELECT * FROM "+name()).types();
	}
	
	std::vector<std::string> indices(void) {
		return dataset().indices(name());
	}
	
	//! @}
	
	//! @name SQL methods
	//!
	//! Convieience methods for executung SQL on dataset
	//! @{
	
	//! Execute SQL but do not return anything
	//!
	//! Used for UPDATE, INSERT etc SQL statements
	void execute(const std::string& sql){
		return dataset().execute(sql);
	}
	
	//!
	Datacursor query(const std::string& sql){
		return dataset().query(sql);
	}
	
	//!
	template<typename Type = std::vector<std::string>>
	std::vector<Type> fetch(const std::string& sql){
		return dataset().fetch<Type>(sql);
	}
	
	//! @todo Add other Dataset method such as value, column etc?
	
	//! @}
};

inline Datatable Dataset::table(std::string name){
	return Datatable(this,name);
}

}