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

void trim(std::string& string);

std::vector<std::string> split(const std::string& string, const std::string& separator);

}
