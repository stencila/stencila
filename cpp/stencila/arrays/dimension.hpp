#pragma once

#include "forwards.hpp"

namespace Stencila {
namespace Arrays {

template<
	class Derived,
	unsigned int Size
>
class Dimension {
public:
	/**
	 * Size, i.e. number of levels
	 */
	static const unsigned int size;

	/**
	 * Text label used for outputting
	 */
	static const char* label;

	/**@{
	 * Iteration over a dimension
	 */

	/**
	 * Iterator class which provides for convenient looping
	 * over levels in a dimension.
	 *
	 * Based on http://stackoverflow.com/a/7185723
	 */
	class Level {
	public:
		unsigned int operator*() const{ 
			return level_;
		}

		const Level& operator++() {
			++level_;
			return *this;
		}

		Level operator++(int){
			Level copy(*this);
			++level_;
			return copy;
		}

		bool operator==(const Level &other) const {
			return level_ == other.level_;
		}

		bool operator!=(const Level &other) const {
			return level_ != other.level_;
		}

	protected:
		Level(unsigned int start):level_(start){}
		friend class Dimension;

	private:
		unsigned int level_;
	};

	Level begin(void) const { 
		return Level(0); 
	}

	Level end() const {
		return Level(Size);
	}

	//!@}
};

// Templated static size and label definitions

template<
	class Derived,
	unsigned int Size
>
const unsigned int Dimension<Derived,Size>::size = Size;

template<
	class Derived,
	unsigned int Size
>
const char* Dimension<Derived,Size>::label;

/**
 * A macro to create an Arrayspace Dimension class.
 *
 * Createing a dimension class by hand can be tedious...
 *
 * class Region : public Dimension<Region,3>{} regions;
 * template<> const char* Dimension<Region,3>::label = "region";
 *
 * This macro lets you replace that with...
 * 
 * STENCILA_ARRAY_DIM(Region,region,10)
 * 
 * @param  name   	Name of dimension (e.g. Region)
 * @param  instance	Name of dimenstion instance (e.g. regions)
 * @param  lab 		Label for dimension (e.g. region)
 * @param  Size 	Number of levels in the dimension (e.g. 32)
 */
#define STENCILA_ARRAY_DIM(name,instance,lab,size) \
	class name : public Dimension<name,size>{} instance; \
	template<> const char* Dimension<name,size>::label = #lab;

/**
 * Singular dimensions are Dimensions with only one level.
 * They are used as default dimensions for Arrays
 */
class Singular1 : public Dimension<Singular1,1>{};
class Singular2 : public Dimension<Singular2,1>{};
class Singular3 : public Dimension<Singular3,1>{};
class Singular4 : public Dimension<Singular4,1>{};
class Singular5 : public Dimension<Singular5,1>{};
class Singular6 : public Dimension<Singular6,1>{};

}
}