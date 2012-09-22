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

//!	@file print.hpp
//!	@brief Classes for printing Stencila, builtin and standard library objects
//!
//!	These classes aim to provide a simple, consitent output interface for use within Stencila.
//!	Objects such as std::strings, std::vectors and std::maps are represented in similar ways as in Python and JSON
//!	There are other C++ libraries that offer similar functionality e.g. http://louisdx.github.com/cxx-prettyprint/

#pragma once

#include <iostream>
#include <sstream>
#include <fstream>

/*
There are several ways to convert fundamental types to string e.g.

std::sprintf
std::stringstream
boost::lexical_cast
boost::print
FastFormat : http://www.fastformat.org/
StrTk : http://www.codeproject.com/Articles/23198/C-String-Toolkit-StrTk-Tokenizer

There are some sedd comparisons including these

http://www.boost.org/doc/libs/1_51_0/doc/html/boost_lexical_cast/performance.html
http://fastformat.sourceforge.net/performance.html

The following macros let you choose between two alternatives that
are used for expediency...
*/
#define STENCILA_PRINT_OSTRINGSTREAM 1
#define STENCILA_PRINT_BOOSTLEXICALCAST 0

#if STENCILA_PRINT_BOOSTLEXICALCAST
	#include <boost/lexical_cast.hpp>
#endif

#include <stencila/traits.hpp>

namespace Stencila {
    
using namespace Traits;
	
template<typename Type>
class Printer;
	
template<>
class Printer<void> {

public:
	//! Any pointer is printed with a ampersand prefix
	//! A null pointer is printed as "null" and other 
	template<typename Type>
	static std::string print(Type* pointer){
		if(pointer){
			return "&"+print(*pointer);
		} else {
			return "&null";
		}
	}
	
	static std::string print(const char& value){
		return std::string("\'")+value+"\'";
	}

	static std::string print(const char* value){
		return std::string("\"")+value+"\"";
	}

	static std::string print(const std::string& value){
		return "\""+value+"\"";
	}
	

	template<class T1, class T2>
	static std::string print(std::pair<T1,T2> const & pair){
		return "("+print(pair.first)+","+print(pair.second)+")";
	}

	//! Print a std::tuple as (x,y,z,...)
	template<typename... Types>
	static std::string print(std::tuple<Types...> const & tuple){
		const unsigned int Size = std::tuple_size<std::tuple<Types...>>::value;
		return "("+ format_tuple_helper<Size,Types...>::help(tuple) +")";
	}

	template <size_t Size, typename... Types>
	struct format_tuple_helper {
	    static std::string help(const std::tuple<Types...>& tuple ) {
		return format_tuple_helper<Size-1,Types...>::help(tuple) + "," + Printer<void>::print(std::get<Size-1>(tuple));
	    }
	};

	template <typename... Types>
	struct format_tuple_helper<1,Types...> {
	    static std::string help(const std::tuple<Types...>&  tuple) {
		return Printer<void>::print(std::get<0>(tuple));
	    }
	};

	template<typename Type>
	static std::string print(Type value){
		return print(IsContainer<Type>(),value);
	}

	template<typename Type>
	static std::string print(std::false_type not_a_container, Type value){
		#if STENCILA_PRINT_BOOSTLEXICALCAST
		return boost::lexical_cast<std::string>(value);
		#elif STENCILA_PRINT_OSTRINGSTREAM
		std::ostringstream stream;
		stream << value;
		return stream.str();
		#endif
	}
	
	template<typename Type>
	static std::string print(const std::true_type& is_container, Type container){
		return print_container(IsAssociative<Type>(),IsPaired<Type>(),container);
	}

	template<typename Type>
	static std::string print_container(const std::false_type& is_set, const std::false_type& is_map, Type container){
		std::ostringstream stream;
		stream << '[';
		auto i = container.begin();
		auto e = container.end();
		if(i!=e){
			while(true){
				stream<<print(*i);
				if(++i==e) break;
				stream<<",";
			}
		}
		stream << ']';
		return stream.str();
	}
    
	template<typename Type>
	static std::string print_container(const std::true_type& is_assoc, const std::false_type& is_paired, Type container){
		
		std::ostringstream stream;
		stream << '{';
		auto i = container.begin();
		auto e = container.end();
		if(i!=e){
			while(true){
				stream<<print(*i);
				if(++i==e) break;
				stream<<",";
			}
		}
		stream << '}';
		return stream.str();
	}

	template<typename Type>
	static std::string print_container(const std::true_type& is_assoc, const std::true_type& is_paired, Type container){
		
		std::ostringstream stream;
		stream << '{';
		auto i = container.begin();
		auto e = container.end();
		if(i!=e){
			while(true){
				stream<<print(i->first)<<":"<<print(i->second);
				if(++i==e) break;
				stream<<",";
			}
		}
		stream << '}';
		return stream.str();
	}

	template<typename Type>
	Printer& operator<<(Type const & value){
		std::cout<<print(value);
		return *this;
	}
	
	Printer& operator<<(const char* const & value){
		std::cout<<value;
		return *this;
	}
	
	Printer<std::ostringstream> operator()(void);
	Printer<std::ostream> operator()(std::ostream& stream);
	Printer<std::ofstream> operator()(const char* filename);
	Printer<std::ofstream> operator()(const std::string& filename);
};

template<>
class Printer<std::ostringstream> : public Printer<void> {
private:	
	std::ostringstream* stream_;

public:
	Printer(void):
		stream_(new std::ostringstream){
	}
	
	~Printer(void){
		delete stream_;
	}
	
	template<typename Type>
	Printer& operator<<(Type const & value){
		*stream_<<print(value);
		return *this;
	}
	
	Printer& operator<<(const char* const & value){
		*stream_<<value;
		return *this;
	}
	
	operator const char*(void){
		return stream_->str().c_str();
	}
	
	operator std::string(void){
		return stream_->str();
	}
};

template<>
class Printer<std::ostream> : public Printer<void> {
private:
	std::ostream* stream_;

public:
		
	Printer(std::ostream& stream):
		stream_(&stream){
	}
	
	template<typename Type>
	Printer& operator<<(Type const & value){
		*stream_<<print(value);
		return *this;
	}
	
	Printer& operator<<(const char* const & value){
		*stream_<<value;
		return *this;
	}
};

template<>
class Printer<std::ofstream> : public Printer<void> {
private:
	std::ofstream* file_;

public:
		
	Printer(const char* filename):
		file_(new std::ofstream(filename)){
	}
	
	Printer(const std::string& filename):
		file_(new std::ofstream(filename.c_str())){
	}
	
	~Printer(void){
		file_->close();
		delete file_;
	}
    
	template<typename Type>
	Printer& operator<<(Type const & value){
		*file_<<print(value);
		return *this;
	}
	
	Printer& operator<<(const char* const & value){
		*file_<<value;
		return *this;
	}
    
	struct Closer {};
	void operator<<(const Closer& closer){
		file_->close();
	}
};

inline Printer<std::ostringstream> Printer<void>::operator()(void){
	return Printer<std::ostringstream>();
}

inline Printer<std::ostream> Printer<void>::operator()(std::ostream& stream){
	return Printer<std::ostream>(stream);
}

inline Printer<std::ofstream> Printer<void>::operator()(const char* filename){
	return Printer<std::ofstream>(filename);
}

inline Printer<std::ofstream> Printer<void>::operator()(const std::string& filename){
	return Printer<std::ofstream>(filename);
}

Printer<void> print;

const char* $ = "\n";
Printer<std::ofstream>::Closer $$;

}