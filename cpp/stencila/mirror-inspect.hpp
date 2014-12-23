#pragma once

#include <vector>

#include <stencila/mirror.hpp>
#include <stencila/traits.hpp>

namespace Stencila {
namespace Mirrors {

class Has : public Mirror<Has> {
public:

	Has(const std::string& name):
		name_(name), has_(false){}

	template<typename Data,typename... Args>
	Has& data(Data& data, const std::string& name, Args... args){
		if(not has_) has_ = name==name_;
		return *this;
	}

	operator bool(void) const {
		return has_;
	}

private:
	std::string name_;
	bool has_;
}; // class Has


class Labels : public Mirror<Labels>, public std::vector<std::string> {
public:

	template<typename Type,typename... Args>
	Labels& data(Type& data, const std::string& name, Args... args){
		data_(data,name,IsStructure<Type>(),IsArray<Type>());
		return *this;
	}

private:
	template<typename Type>
	Labels& data_(Type& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		prefix_ = name + ".";
		data.reflect(*this);
		prefix_ = "";
		return *this;
	}

	template<typename Type>
	Labels& data_(Type& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		prefix_ = name;
		data.reflect(*this);
		prefix_ = "";
		return *this;
	}

	template<typename Type>
	Labels& data_(Type& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		push_back(prefix_+name);
		return *this;
	}

	std::string prefix_;
}; // class Labels

} // namepace Mirrors
} // namespace Stencila

