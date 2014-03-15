#pragma once

namespace Stencila {

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

	/**
	 * Construct a level from an integer.
	 *
	 */
	Level(unsigned int level):
		level_(level){
	}

	/**
	 * Implicit conversion to an `unsigned int`.
	 */
	operator unsigned int (void) const{ 
		return level_;
	}

	/**
	 * Dereference.
	 *
	 * Returns a copy, instead of an `unsigned int`, because Level<Dimension>
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
	static const unsigned int size(void) {
		return Size;
	}

	/**
	 * Implicit conversion to an unsigned int for syntactic
	 * convienience
	 */
	operator unsigned int (void) const { 
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
	 * Calling operator to get a level for this dimension
	 */
	Level<Derived> operator()(const unsigned int& index) const { 
		return Level<Derived>(index); 
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
		static const char* label(void) { return "singular"; } \
	};

STENCILA_DIM_SINGULAR(Singular1)
STENCILA_DIM_SINGULAR(Singular2)
STENCILA_DIM_SINGULAR(Singular3)
STENCILA_DIM_SINGULAR(Singular4)
STENCILA_DIM_SINGULAR(Singular5)
STENCILA_DIM_SINGULAR(Singular6)
STENCILA_DIM_SINGULAR(Singular7)
STENCILA_DIM_SINGULAR(Singular8)
STENCILA_DIM_SINGULAR(Singular9)
STENCILA_DIM_SINGULAR(Singular10)

#undef STENCILA_DIM_SINGULAR

} //namespace Stencila
