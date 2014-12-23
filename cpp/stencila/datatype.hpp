#pragma once

#include <string>
#include <typeinfo>
#include <cstdlib>
#include <memory>
#include <cxxabi.h>

namespace Stencila {

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

	operator std::string(void){
		return name();
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

	/**
	 * Demangle a g++ type name
	 */
	static std::string demangle(const char* name) {
		// From http://stackoverflow.com/a/4541470 with thanks
		int status = -4;
		std::unique_ptr<char, void(*)(void*)> result {
			abi::__cxa_demangle(name, NULL, NULL, &status),
			std::free
		};
		return (status==0) ? result.get() : name ;
	}

	static Datatype from_type_info(const std::type_info& type);
};

static const Datatype Null('n');
static const Datatype Integer('i');
static const Datatype Real('r');
static const Datatype Text('t');

Datatype Datatype::from_type_info(const std::type_info& type) {
	if(type==typeid(void)) return Null;
	if(type==typeid(int)) return Integer;
	if(type==typeid(float)) return Real;
	if(type==typeid(double)) return Real;
	if(type==typeid(std::string)) return Text;
	STENCILA_THROW(Exception,str(boost::format("Unrecognised type <%s>")%demangle(type.name())));
}

}
