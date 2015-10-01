#define STENCILA_STRING_CPP

#include <iostream>       // std::cout
#include <string>         // std::string
#include <locale>         // std::locale, std::tolower

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/regex.hpp>
#include <boost/algorithm/string/case_conv.hpp>

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

std::string trim(const std::string& string){
	return boost::trim_copy(string);
}

std::string lower(const std::string& string){
	return boost::to_lower_copy(string);
}

std::string upper(const std::string& string){
	return boost::to_upper_copy(string);
}

std::string title(const std::string& string, const std::vector<std::string>& exceptions){
	// Thanks to https://www.reddit.com/r/dailyprogrammer/comments/wjzly/7132012_challenge_76_easy_title_case/c5e5a64
    std::string result = boost::to_lower_copy(string);
    std::vector<std::string> words;
    boost::split(words, result, boost::is_any_of(" "));
    for(std::string& word : words) {
        if(std::find(exceptions.begin(), exceptions.end(), word) == exceptions.end()) {
            word[0] = std::toupper(word[0]);
        }
    }
    result = boost::join(words, " ");
    result[0] = std::toupper(result[0]);
    return result;
}

std::string title(const std::string& string){
	return title(string,{});
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