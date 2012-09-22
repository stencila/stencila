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

//! @file datatypes.hpp
//! @brief Definition of data types

#pragma once

namespace Stencila {
	
//! @class Datatype
//! @todo Document fully
class Datatype {
	
public:
	char code;

	Datatype(char value=0):
		code(value){
	}
	
	bool operator==(const Datatype& other) const {
		return code==other.code;
	}
	
	bool operator!=(const Datatype& other) const {
		return code!=other.code;
	}
		
	std::string name(void) const {
		switch(code){
			case 'n': return "Null";
			case 'i': return "Integer";
			case 'r': return "Real";
			case 't': return "Text";
		}
		return "Undefined";
	}
	
	std::string sql(void) const {
		switch(code){
			case 'n': return "NULL";
			case 'i': return "INTEGER";
			case 'r': return "REAL";
			case 't': return "TEXT";
		}
		return "NULL";
	}
};

const Datatype Null('n');
const Datatype Integer('i');
const Datatype Real('r');
const Datatype Text('t');

}
