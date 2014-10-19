#pragma once

namespace Stencila {
	
std::string string(bool value);
std::string string(int value);
std::string string(unsigned int value);
std::string string(long unsigned int value);
std::string string(float value);
std::string string(double value);
std::string string(const std::string& value);

template<typename Type>
Type unstring(const std::string& value);

std::string& trim(std::string& string);

std::string& replace_all(std::string& string, const std::string& what, const std::string& with);

std::vector<std::string> split(const std::string& string, const std::string& separator);

std::string join(const std::vector<std::string>& vector, const std::string& separator);

}
