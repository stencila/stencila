#define STENCILA_STRING_CPP

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/regex.hpp>

#include <stencila/string.hpp>

namespace Stencila {

#define UNSTRING(TYPE_) \
	template<> \
	TYPE_ unstring(const std::string& value){ \
		return boost::lexical_cast<TYPE_>(value); \
	}

UNSTRING(bool)
UNSTRING(char)
UNSTRING(unsigned char)
UNSTRING(int)
UNSTRING(long int)
UNSTRING(unsigned int)
UNSTRING(unsigned long int)
UNSTRING(float)
UNSTRING(double)
UNSTRING(std::string)

#undef UNSTRING

std::string& trim(std::string& string){
	boost::trim(string);
	return string;
}

std::string& replace_all(std::string& string, const std::string& what, const std::string& with){
	boost::replace_all(string,what,with);
	return string;
}

std::vector<std::string> split(const std::string& string, const std::string& separator){
	std::vector<std::string> bits;
	boost::split(bits,string,boost::is_any_of(separator));
	return bits;
}

std::string join(const std::vector<std::string>& vector, const std::string& separator){
	return boost::join(vector,separator);
}

std::string slugify(const std::string& string, unsigned int length){
	std::string copy = string;
	boost::trim(copy);
	boost::to_lower(copy);
	boost::replace_all_regex(
		copy,
		boost::regex("[^a-z0-9]|(\\s+)"),
		std::string("-"),
		boost::match_default | boost::format_all
	);
	if(copy.length()>length) return copy.substr(0,length);
	else return copy;
}

}