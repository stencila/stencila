#include <boost/lexical_cast.hpp>

#include <stencila/string.hpp>

namespace Stencila {

#define STRING(TYPE_) \
	std::string string(TYPE_ value){ \
		return boost::lexical_cast<std::string>(value); \
	}

STRING(bool)
STRING(int)
STRING(unsigned int)
STRING(float)
STRING(const double&)
STRING(const std::string&)

#undef STRING

#define UNSTRING(TYPE_) \
	template<> \
	TYPE_ unstring(const std::string& value){ \
		return boost::lexical_cast<TYPE_>(value); \
	}

UNSTRING(bool)
UNSTRING(int)
UNSTRING(unsigned int)
UNSTRING(float)
UNSTRING(double)
UNSTRING(std::string)

#undef UNSTRING

void trim(std::string& string){
	boost::trim(string);
}

std::vector<std::string> split(const std::string& string, const std::string& separator){
	std::vector<std::string> bits;
	boost::split(bits,string,boost::is_any_of(separator));
	return bits;
}

}