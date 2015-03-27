#pragma once

#include <stencila/polymorph.hpp>

namespace Stencila {
namespace Mirrors {

template<class Derived>
class Mirror : public Polymorph<Derived> {
public:
	using Polymorph<Derived>::derived;

	template<class Type>
	Derived& mirror(void){
		return mirror(*static_cast<Type*>(nullptr));
	}

	template<class Type>
	Derived& mirror(Type& object){
		derived().start(object);
		object.reflect(derived());
		derived().finish(object);
		return derived();
	}

	template<typename Type>
	Derived& start(Type& object){
		return derived();
	}

	template<typename Type>
	Derived& finish(Type& object){
		return derived();
	}

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
