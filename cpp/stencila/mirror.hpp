#pragma once

#include <stencila/polymorph.hpp>

namespace Stencila {
namespace Mirrors {

template<class Derived>
class Mirror : public Polymorph<Derived> {
public:
    using Polymorph<Derived>::derived;

	template<typename Data,typename... Args>
	Derived& data(Data& data, Args... args){
		return derived();
	}

	template<typename Method,typename... Args>
	Derived& method(Method& method, Args... args){
		return derived();
	}
	
}; // class Mirror

} // namepace Mirrors
} // namespace Stencila
