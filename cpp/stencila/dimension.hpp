#pragma once

#include <boost/lexical_cast.hpp>

#include <stencila/traits.hpp>

namespace Stencila {

// Declaration of Dimension class
template<
	class Derived = void,
	uint Size = 0,
	uint Base = 0,
	uint Step = 1
>
class Dimension;

/**
 * A level of a Dimension.
 * 
 * Levels represent a particular index of a dimension. They are used to size, slice and dice a Grid.
 * They act as an iterator for convenient looping over levels in a dimension
 */
template<class Dimension>
class Level {
private:

	/**
	 * The index of the dimension referred to by this level.
	 */
	uint index_;

public:

	/**
	 * Construct a level from an integer label of the dimension.
	 */
	Level(uint label):
		index_(Dimension::level(label).index()){
	}

	/**
	 * Construct a level from an string label of the dimension.
	 */
	Level(const std::string& label):
		index_(Dimension::level(label).index()){
	}

	/**
	 * Construct a level from a level of another dimension.
	 *
	 * It may be unsafe to do this if the size of the other dimension
	 * differs to the size of this one. For that reason this constructor is
	 * made explicit and checking may be implemened at a later stage.
	 * By not having this implicit, the compiler warns if a grid is subscripted with
	 * dimensions in the incorrect order.
	 */
	template<class Other>
	explicit Level(Level<Other> level):
		index_(level.index()){
	}

	/**
	 * Construct a "null" level from an index of the dimension.
	 *
	 * Intended to only be called by a Dimension.
	 */
	explicit Level():
		index_(0){
	}

	/**
	 * Construct a level from an index of the dimension.
	 *
	 * Intended to only be called by a Dimension.
	 * The unused argument prevents abiguity with constructor from uint label
	 */
	explicit Level(uint index,const char* unused):
		index_(index){
	}

	/**
	 * Return the index of the dimension
	 */
	uint index(void) const { 
		return index_;
	}

	/**
	 * Get the label for this level
	 */
	std::string label(void) const { 
		return Dimension::label(index_);
	}

	/**
	 * Dereference operator
	 *
	 * Returns a copy, instead of an `uint`, because Level<Dimension>
	 * is used as an argument to subscript a Grid with this dimension
	 */
	Level<Dimension> operator*() const { 
		return Level<Dimension>(index_);
	}

	/**
	 * @name Increment operators
	 * @{
	 */

	const Level& operator++() {
		++index_;
		return *this;
	}

	Level operator++(int){
		Level copy(*this);
		++index_;
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
		return index_ == other.index_;
	}

	bool operator!=(const Level<Dimension>& other) const {
		return index_ != other.index_;
	}

	bool operator<(const Level<Dimension>& other) const {
		return index_ < other.index_;
	}

	bool operator>(const Level<Dimension>& other) const {
		return index_ > other.index_;
	}

	/**
	 * @}
	 */
};

template<class Dimension>
std::ostream& operator<<(std::ostream& stream, const Level<Dimension>& level){
	stream<<level.label();
	return stream;
}

/**
 * Base class for all dimensions.
 * 
 * Having this base class allows dimensions to be used dynamically 
 * by Array classes.
 */
template<>
class Dimension<> {
private:

	/**
	 * The size of the dimension
	 */
	uint size_;

	/**
	 * The name of the dimension
	 */
	const char* name_;

public:

	/**
	 * Construct a Dimension.
	 *
	 * This is the only constructor since size and name
	 * must always be intialised.
	 */
	Dimension(uint size, const char* name):
		size_(size),
		name_(name){
	}

	/**
	 * Get the size of the dimension.
	 */
	uint size(void) const {
		return size_;
	}	

	/**
	 * Get the name of the dimension.
	 */
	const char* name(void) const {
		return name_;
	}

};

/**
 * A static Dimension class
 */
template<
	class Derived,
	uint Size,
	uint Base,
	uint Step
>
class Dimension : public Dimension<> {
public:

	/**
	 * Construct a dimension.
	 *
	 * This need sto be called so that size and name member data attributes
	 * can be initialised
	 */
	Dimension(const char* name):
		Dimension<>(Size,name){
	}

	/**
	 * Size of dimension.
	 *
	 * A static member that can be used by Grids.
	 * For that reason made public but use of `size()` method should be
	 * preferred.
	 */
	static const uint size_ = Size;

	/**
	 * Size, i.e. number of levels, of dimension
	 *
	 * For consistency with `name()` this is made a static method.
	 * Does not need to be overidden.
	 */
	static uint size(void) {
		return Size;
	}

	/**
	 * Get name of dimension
	 *
	 * This is a static method, rather than a static member, so that derived Dimensions
	 * can be defined within functions (static members can't).
	 * Should be overidden by Derived class
	 */
	static const char* name(void) {
		return "dimension";
	}

	/**
	 * Get a label for an index of this dimension
	 */
	static std::string label(const uint& index) {
		return boost::lexical_cast<std::string>(Base+index*Step);
	}

	/**
	 * Get a "null" level for this dimension. Intended for use in
	 * grids which do not contain this dimension.
	 */
	static Level<Derived> level(void) { 
		return Level<Derived>(); 
	}

	/**
	 * Get a level for an index of this dimension
	 */
	static Level<Derived> level(const uint& label) { 
		uint index = (label-Base)/Step;
		return Level<Derived>(index,"index"); 
	}

	/**
	 * Get a level for a string label
	 *
	 * Currently, only string representations of integers are implemeted.
	 * In the future, text labels will also be allowed.
	 */
	static Level<Derived> level(const std::string& label) {
		return level(boost::lexical_cast<uint>(label));
	}

	/**
	 * Get an index for a label by reading in from a stream
	 */
	static Level<Derived> level(std::istream& stream) {
		std::string label;
		stream>>label;
		return level(label);
	}

	/**
	 * Begin iterator, a level associated with the 0 index
	 */
	Level<Derived> begin(void) const { 
		return Level<Derived>(0,"index"); 
	}

	/**
	 * End iterator, a level associated with the last index plus one
	 */
	Level<Derived> end(void) const {
		// Note that this is intended to use `Size` instead of `Size-1` since the is returning `end` not `last`.
		return Level<Derived>(Size,"index");
	}
};

/**
 * A macro to create a Dimension class.
 *
 * Creating a dimension class by hand can be tedious...
 *
 *     struct Region : Dimension<Region,3>{
 *     		const char* name(void) const { return "region"; }
 *     } regions;
 *
 * This macro lets you replace that with...
 * 
 *     STENCILA_DIM(Region,regions,region,3)
 * 
 * @param  class_   	Class name for dimension (e.g. Region)
 * @param  instance		Name of dimenstion instance (e.g. regions)
 * @param  name_ 		Name for dimension (e.g. region)
 * @param  size 		Number of levels in the dimension (e.g. 3)
 */
#define STENCILA_DIM(class_,instance,name_,size) \
	class class_ : public Dimension<class_,size> { \
	public: \
		class_(void):Dimension<class_,size>(#name_){} \
		static const char* name(void) { return #name_; } \
	} instance;

/**
 * Singular dimensions.
 * Dimensions with only one level used as default dimensions for Arrays
 */
#define STENCILA_DIM_SINGULAR(class_) \
	class class_ : public Dimension<class_,1> { \
	public: \
		class_(void):Dimension<class_,1>("singular"){} \
		static const char* name(void) { return "singular"; } \
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
