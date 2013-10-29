#pragma once

#include "dimension.hpp"

namespace Stencila {
namespace Arrayspace {

template<
	class Type = double,
	class Dimension1 = Singular1,
	class Dimension2 = Singular2,
	class Dimension3 = Singular3,
	class Dimension4 = Singular4,
	class Dimension5 = Singular5,
	class Dimension6 = Singular6
>
class Array;


template<
	class Derived
>
class Aggregator {
public:
	
	const Derived& derived(void) const {
		return static_cast<const Derived&>(*this);
	}

	Derived& derived(void) {
		return static_cast<Derived&>(*this);
	}

	template<class Type>
	void append(const Type& value){
	}

	double finalise(void){
		return 0;
	}

	template<class Array>
	double aggregate(const Array& array) {
		for(auto& value : array) derived().append(value);
		return derived().finalise();
	}

	template<
		typename Type,
		class... Dims,
		typename Return,
		typename Arg
	>
	double aggregate(const Array<Type,Dims...>& array, Return (*function)(Arg)) {
		for(auto& value : array) derived().append(function(value));
		return derived().finalise();
	}
};

class Count : public Aggregator<Count> {
private:
	double count_;
	
public:
	Count(void):
		count_(0){
	}

	void append(void){
		count_++;
	}

	template<class Type>
	void append(const Type& value){
		count_++;
	}

	double finalise(void){
		return count_;
	}
};

Count count(){
	return Count();
}

class Sum : public Aggregator<Sum> {
private:
	double sum_;

public:

	Sum(void):
		sum_(0){
	}

	template<class Type>
	void append(const Type& value){
		sum_ += value;
	}

	double finalise(void){
		return sum_;
	}
};

Sum sum(){
	return Sum();
}

template<
	typename Type,
	class... Dims
>
double sum(const Array<Type,Dims...>& array){
	return Sum().aggregate(array);
}

template<
	typename Return,
	typename Arg
>
double sum(Return(*function)(Arg)){
	return Sum().aggregate(function);
}

/*
template<
	class Dims...,
	typename Return,
	typename Args...
>
class Product {
private:

	Return (*function_)(Args...);
	double product_;
	
public:

	Product(Return (*function)(Args...)):
		function_(function),
		product_(1){
	}

	operator double(void){
		return aggregate();
	}

	template<class Type>
	void append(const Type& value){
		product_ *= value;
	}

	double finalise(void){
		return product_;
	}
};

template<
	class... Dims,
	typename Return,
	typename... Args
>
Product product(
	Dims...,
	Return (*function)(Args...)
){
	return Product<Dims...,Return,Args...>(function);
}
*/

/*
class Mean : public Aggregator<Mean> {
private:
	double sum_;
	double count_;

public:

	template<class Type>
	void append(const Type& value){
		sum += value_;
		Count::append();
	}

	double finalise(void){
		return Sum::finalise()/Count::finalise();
	}
};

Mean mean(){
	return Mean();
}

class Geomean : public Sum, public Count {
public:

	template<class Type>
	void append(const Type& value){
		if(value>0){
			Sum::append(std::log(value));
			Count::append();
		}
	}

	double finalise(void){
		return std::exp(Sum::finalise()/Count::finalise());
	}
};

Geomean geomean(){
	return Geomean();
}
*/

template<
	typename Function
>
class Func : public Aggregator<Func<Function>> {
private:
	Function func_;

public:

	Func(Function func):
		func_(func){}

	template<class Type>
	void append(const Type& value){
		func_(value);
	}
};

template<
	typename Function
>
Func<Function> func(Function func){
	return Func<Function>(func);
}

template<
	class Dim1 = Singular1,
	class Dim2 = Singular2,
	class Dim3 = Singular3,
	class Dim4 = Singular4,
	class Dim5 = Singular5,
	class Dim6 = Singular6
>
class By {
public:
};

template<
	class... Dims
>
By<Dims...> by(Dims... dims){
	return By<Dims...>();
}

}
}
