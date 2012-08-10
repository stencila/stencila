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

//!	@file printing.hpp
//!	@brief Functions for printing Stencila, builtin and standard library objects
//!
//!	These functions aim to provide a consitent output interface for use within Stencila.
//!	Objects such as std::strings, std::vectors and std::maps are represented in similar ways as in Python and JSON

#pragma once

#include <iostream>
#include <string>
#include <vector>
#include <tuple>

namespace Stencila {
	
template<typename Value>	
void print_raw(const Value& value){
	std::cout<<value;
}
	
template<typename Value, typename... Values>	
void print_raw(const Value& value,const Values&... values){
	print_raw(value);
	print_raw(values...);
}

void print_flush(void){
	std::cout<<std::flush;
}
	
template<typename Type>	
void print_format(const Type& value){
	print_raw(value);
}

template<>
void print_format(const char& character){
	print_raw('\'',character,'\'');
}

template<>
void print_format(const std::string& string){
	print_raw('"',string,'"');
}

template<typename Type>
void print_format(const std::vector<Type>& vector){
	print_raw("[");
	for(auto item=vector.begin();item!=vector.end();item++){
		print_format(*item);
		if(item!=vector.end()-1) print_raw(", ");
	}
	print_raw("]");
}

void print_attrs(void){
}

template<typename Value>	
void print_attrs(const char* name, const Value& value){
	print_raw(name,":");
	print_format(value);
}

template<typename Value, typename... Attrs>	
void print_attrs(const char* name, const Value& value, const Attrs&... attrs){
	print_attrs(name,value);
	print_raw(", ");
	print_attrs(attrs...);
}

template<typename Object, typename... Attrs>	
void print_object(const char* type, const Object& object, const Attrs&... attrs){
	print_raw(type,"@",&object,"{ ");
	print_attrs(attrs...);
	print_raw(" }");
}
	
template<typename Type>	
void print(const Type& value){
	print_format(value);
	print_raw("\n");
	print_flush();
}

}