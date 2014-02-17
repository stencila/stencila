#pragma once

#include <stencila/traits.hpp>

namespace Stencila {

template<class Dimension> class Level;

/**
 * A level of an array Dimension.
 * 
 * Implements an iterator interface for convenient looping
 * over levels in a dimension (based on [this](http://stackoverflow.com/a/7185723))
 */
template<class Dimension>
class Level {
private:
	unsigned int level_;

public:
	Level(unsigned int start):
		level_(start){
	}

	/**
	 * Convert to an unsigned int
	 */
	operator unsigned int (void) const{ 
		return level_;
	}

	/**
	 * Dereference.
	 *
	 * Returns a copy, instead of an unsigned int, because Level<Dimension>
	 * is used an argument to index an Array<Dimension,...>
	 */
	unsigned int operator*() const{ 
		return Level<Dimension>(level_);
	}

	/**
	 * @name Increment operators
	 * @{
	 */

	const Level& operator++() {
		++level_;
		return *this;
	}

	Level operator++(int){
		Level copy(*this);
		++level_;
		return copy;
	}

	/**
	 * @}
	 */


	/**
	 * @name Comparison operators
	 * @{
	 */

	bool operator==(const Level<Dimension>& other) const {
		return level_ == other.level_;
	}

	bool operator==(const unsigned int& other) const {
		return level_ == other;
	}

	bool operator!=(const Level<Dimension>& other) const {
		return level_ != other.level_;
	}

	bool operator!=(const unsigned int& other) const {
		return level_ != other;
	}

	/**
	 * @}
	 */
};

template<
	class Derived,
	unsigned int Size
>
class Dimension {
public:

	/**
	 * Size of dimension.
	 *
	 * A static member that can be used in definition of Arrays.
	 * For that reason made public but use of `size()` method should be
	 * preferred.
	 */
	static const unsigned int size_ = Size;

	/**
	 * Size, i.e. number of levels, of dimension
	 *
	 * For consistency with `label()` this is made a static method.
	 * Does not need to be overidden.
	 */
	static const unsigned int size(void){
		return Size;
	}

	/**
	 * Text label used when writing an Array to output
	 *
	 * This is a static method, rather than a static member, so that derived Dimensions
	 * can be defined within functions (static members can't).
	 * Should be overidden by Derived class
	 */
	static const char* label(void) {
		return "dimension";
	}

	/**
	 * Begin iterator
	 */
	Level<Derived> begin(void) const { 
		return Level<Derived>(0); 
	}

	/**
	 * End iterator
	 */
	Level<Derived> end(void) const {
		return Level<Derived>(Size);
	}
};

/**
 * A macro to create a Dimension class.
 *
 * Creating a dimension class by hand can be tedious...
 *
 *     struct Region : Dimension<Region,3>{
 *     		const char* label(void) const { return "region"; }
 *     } regions;
 *
 * This macro lets you replace that with...
 * 
 *     STENCILA_DIM(Region,regions,region,3)
 * 
 * @param  name   	Name of dimension (e.g. Region)
 * @param  instance	Name of dimenstion instance (e.g. regions)
 * @param  lab 		Label for dimension (e.g. region)
 * @param  size 	Number of levels in the dimension (e.g. 3)
 */
#define STENCILA_DIM(name,instance,lab,size) \
	class name : public Dimension<name,size>{ \
	public: \
		static const char* label(void) { return #lab; } \
	} instance;

/**
 * Singular dimensions.
 * Dimensions with only one level used as default dimensions for Arrays
 */
#define STENCILA_DIM_SINGULAR(name) \
	class name : public Dimension<name,1>{ \
	public: \
		static const char* label(void) { return "single"; } \
	};

STENCILA_DIM_SINGULAR(Single1)
STENCILA_DIM_SINGULAR(Single2)
STENCILA_DIM_SINGULAR(Single3)
STENCILA_DIM_SINGULAR(Single4)
STENCILA_DIM_SINGULAR(Single5)
STENCILA_DIM_SINGULAR(Single6)
STENCILA_DIM_SINGULAR(Single7)
STENCILA_DIM_SINGULAR(Single8)
STENCILA_DIM_SINGULAR(Single9)
STENCILA_DIM_SINGULAR(Single10)

#undef STENCILA_DIM_SINGULAR


template<
	typename Type = double,
	class D1 = Single1,
	class D2 = Single2,
	class D3 = Single3,
	class D4 = Single4,
	class D5 = Single5,
	class D6 = Single6,
	class D7 = Single7,
	class D8 = Single8,
	class D9 = Single9,
	class D10 = Single10
>
class Array;


template<
	typename Type,
	class D1, class D2,	class D3, class D4, class D5,
	class D6, class D7,	class D8, class D9, class D10
>
class Array {
private:

	static const unsigned int size_ =
		D1::size_ * D2::size_ * D3::size_ * D4::size_ * D5::size_ *
		D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;

	Type values_[size_];

public:

	/**
	 * @namespace COnstructors
	 * @{
	 */

	Array(void){
	}

	Array(const Type& other){
		for(Type& value : values_) value = other;
	}

	template<class Other>
    Array(const Other& other){
    	construct_(Traits::IsContainer<Other>(),other);
    }

	/**
	 * Construct from an initializer_list (e.g. `{1.23,3.14,5.98}`)
	 *
	 * This constructor appears to be nessary because compiler (gcc 4.81 at least)
	 * can not resolve between above consturtors when called with an intiializer list
	 */
    template<class Value>
	Array(const std::initializer_list<Value>& values){
        construct_(std::true_type(),values);
    }

private:

 	template<class Other>
    construct_(const std::false_type& is_not_container,const Other& other){
        for(Type& value : values_) value = other;for(Type& value : values_) value = other;
    }

 	template<class Other>
    construct_(const std::true_type& is_container,const Other& other){
        uint index = 0;
        for(auto& item : other){
            values_[index] = item;
            index++;
            if(index>=size_) break;
        }
    }

    /**
     * @}
     */

public:

	/**
	 * Size of Array.
	 */
    static unsigned int size(void) {
		return size_;
	}

	/**
	 * @name Subscript operators
	 *
	 * Return the value at the linear index
	 * 
	 * @{
	 */

	Type& operator[](unsigned int index){
		return values_[index];
	}

	const Type& operator[](unsigned int index) const {
		return values_[index];
	}

	/**
	 * @}
	 */

};

}
