#pragma once

#include <stencila/mirror.hpp>

namespace Stencila {
namespace Mirrors {

class Has : public Mirror<Has> {
private:
    std::string name_;
    bool has_;

public:

    template<class Type>
    Has(const Type& type, const std::string& name):
        name_(name),
        has_(false){
        static_cast<Type*>(nullptr)->reflect(*this);
    }

 	template<typename Data,typename... Args>
	Has& data(Data& data, const std::string& name, Args... args){
		if(not has_) has_ = name==name_;
		return *this;
	}

	operator bool(void) const {
		return has_;
	}
  
}; // class Has

} // namepace Mirrors
} // namespace Stencila

