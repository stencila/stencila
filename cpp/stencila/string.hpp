#pragma once

#include <sstream>

namespace Stencila {
	
/**
 * Convert a value into a string
 *
 * This uses the std::ostream formatting rather than 
 * boost::lexical_cast so that the string is more human readable.
 */
template<typename Type>
std::string string(const Type& value){
	std::stringstream stream;
	stream<<value;
	return stream.str();
}

/**
 * Convert a string into a value of another type
 *
 * Uses boost::lexical_cast for conversion.
 */
template<typename Type>
Type unstring(const std::string& value);

/**
 * Remove all leading and trailing spaces from a string
 */
std::string& trim(std::string& string);

/**
 * Replace all occurrences of `what` in `string` with `with`
 */
std::string& replace_all(std::string& string, const std::string& what, const std::string& with);

/**
 * Split string into a vector of strings using a separator
 */
std::vector<std::string> split(const std::string& string, const std::string& separator);

/**
 * Join a vector of strings into a single string using a separator
 */
std::string join(const std::vector<std::string>& vector, const std::string& separator);

}
