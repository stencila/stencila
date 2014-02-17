#pragma once

#include <fstream>

#include <boost/filesystem.hpp>
#include <boost/preprocessor/seq/for_each.hpp>

#include <stencila/exception.hpp>
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


/**
 * A cell of an array.
 * 
 * Implements an iterator interface for convenient looping
 * over levels in a dimension (based on [this](http://stackoverflow.com/a/7185723))
 */
template<class Type>
class Cell {
private:
	const Type* value_;

public:
	Cell(const Type* value):
		value_(value){
	}

	/**
	 * Dereference.
	 */
	const Type& operator*() const { 
		return *value_;
	}

	/**
	 * @name Increment operators
	 * @{
	 */

	Cell& operator++() {
		++value_;
		return *this;
	}

	Cell operator++(int){
		Cell copy(*this);
		++value_;
		return copy;
	}

	/**
	 * @}
	 */


	/**
	 * @name Comparison operators
	 * @{
	 */

	bool operator==(const Cell<Type>& other) const {
		return value_ == other.value_;
	}

	bool operator!=(const Cell<Type>& other) const {
		return value_ != other.value_;
	}

	/**
	 * @}
	 */
};

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


/**
 * A static array
 *
 * An array with fixed, known dimensions
 */
template<
	typename Type,
	class D1, class D2,	class D3, class D4, class D5,
	class D6, class D7,	class D8, class D9, class D10
>
class Array {
private:

	// A sequence of dimension numbers used below for application
	// of [BOOST_PP_SEQ_FOR_EACH](http://www.boost.org/doc/libs/1_55_0/libs/preprocessor/doc/ref/seq_for_each.html)
	#define STENCILA_ARRAY_DIMENSIONS (D1)(D2)(D3)(D4)(D5)(D6)(D7)(D8)(D9)(D10)

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
    	construct_(IsContainer<Other>(),other);
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
        for(Type& value : values_) value = other;
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
	 * Get the size of the array.
	 */
    static unsigned int size(void) {
		return size_;
	}

 	/**
	 * @name Iterator interface
	 *
	 * @{
	 */

	Cell<Type> begin(void) const {
		return Cell<Type>(&values_[0]);
	}

	Cell<Type> end(void) const {
		return Cell<Type>(&values_[size_]);
	}    

    /**
     * @}
     */
    

	/**
	 * Does the array have a dimension?
	 */
	template<class Dimension>
	static bool dimensioned(const Dimension&) {
		return false;
	}

	#define STENCILA_ARRAY_DIMENSIONED(r,data,elem)\
		static bool dimensioned(const elem&) { return true; }
	BOOST_PP_SEQ_FOR_EACH(STENCILA_ARRAY_DIMENSIONED, , STENCILA_ARRAY_DIMENSIONS)

	#undef STENCILA_ARRAY_DIMENSIONED

	/**
	 * Get the number of cells in a single level of a dimension
	 */
	template<class Dimension>
	static unsigned int base(const Dimension&) { 
		return 0;
	}
	static unsigned int base(const D1&) { 
		return D2::size_ * D3::size_ * D4::size_ * D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D2&) { 
		return D3::size_ * D4::size_ * D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D3&) { 
		return D4::size_ * D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D4&) { 
		return D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D5&) { 
		return D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D6&) { 
		return D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D7&) { 
		return D8::size_ * D9::size_ * D10::size_;
	}
	static unsigned int base(const D8&) { 
		return D9::size_ * D10::size_;
	}
	static unsigned int base(const D9&) { 
		return D10::size_;
	}
	static unsigned int base(const D10&) { 
		return 1;
	}

	/**
	 * Get the level of a dimension at a linear index
	 * 
	 * @param  dimension  The dimension
	 * @param  index The linear index
	 */
	template<class Dimension>
	unsigned int level(Dimension dimension, unsigned int index) const {
		return 0;
	}
	unsigned int level(D1 dimension, unsigned int index) const {
		return index/base(dimension);
	}
	unsigned int level(D2 dimension, unsigned int index) const {
		return index/base(dimension)%D2::size_;
	}
	unsigned int level(D3 dimension, unsigned int index) const {
		return index/base(dimension)%D3::size_;
	}
	unsigned int level(D4 dimension, unsigned int index) const {
		return index/base(dimension)%D4::size_;
	}
	unsigned int level(D5 dimension, unsigned int index) const {
		return index/base(dimension)%D5::size_;
	}
	unsigned int level(D6 dimension, unsigned int index) const {
		return index/base(dimension)%D6::size_;
	}


	/**
	 * Get the linear index corresponding to particular levels of each 
	 * of the array's dimensions
	 */
	static unsigned int index(
		const Level<D1>& l1,
		const Level<D2>& l2 = Level<Single2>(0),
		const Level<D3>& l3 = Level<Single3>(0),
		const Level<D4>& l4 = Level<Single4>(0),
		const Level<D5>& l5 = Level<Single5>(0),
		const Level<D6>& l6 = Level<Single6>(0),
		const Level<D7>& l7 = Level<Single7>(0),
		const Level<D8>& l8 = Level<Single8>(0),
		const Level<D9>& l9 = Level<Single9>(0),
		const Level<D10>& l10 = Level<Single10>(0)
	) {
		return 
			l1 * base(D1()) + 
			l2 * base(D2()) +
			l3 * base(D3()) +
			l4 * base(D4()) +
			l5 * base(D5()) +
			l6 * base(D6()) +
			l7 * base(D7()) +
			l8 * base(D8()) +
			l9 * base(D9()) +
			l10
		;
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

	Type& operator()(
		const Level<D1>& l1,
		const Level<D2>& l2 = Level<Single2>(0),
		const Level<D3>& l3 = Level<Single3>(0),
		const Level<D4>& l4 = Level<Single4>(0),
		const Level<D5>& l5 = Level<Single5>(0),
		const Level<D6>& l6 = Level<Single6>(0),
		const Level<D7>& l7 = Level<Single7>(0),
		const Level<D8>& l8 = Level<Single8>(0),
		const Level<D9>& l9 = Level<Single9>(0),
		const Level<D10>& l10 = Level<Single10>(0)
	){
		return values_[index(l1,l2,l3,l4,l5,l6,l7,l8,l9,l10)];
	}

	const Type& operator()(
		const Level<D1>& l1,
		const Level<D2>& l2 = Level<Single2>(0),
		const Level<D3>& l3 = Level<Single3>(0),
		const Level<D4>& l4 = Level<Single4>(0),
		const Level<D5>& l5 = Level<Single5>(0),
		const Level<D6>& l6 = Level<Single6>(0),
		const Level<D7>& l7 = Level<Single7>(0),
		const Level<D8>& l8 = Level<Single8>(0),
		const Level<D9>& l9 = Level<Single9>(0),
		const Level<D10>& l10 = Level<Single10>(0)
	) const {
		return values_[index(l1,l2,l3,l4,l5,l6,l7,l8,l9,l10)];
	}

	/**
	 * @}
	 */

	/**
	 * Write array to an output stream
	 * 
	 * @param stream Output stream
	 * @param format Format specifier string (e.g. "tsv", "csv")
	 *
	 * @todo Implement more output formats including tuning off header and binary output
	 */
	void write(std::ostream& stream,const std::string format="tsv") const {
		if(format=="tsv"){
			// Header
			#define STENCILA_ARRAY_HEADER(r,data,elem) if(elem::size_>1) stream<<elem::label()<<"\t";
			BOOST_PP_SEQ_FOR_EACH(STENCILA_ARRAY_HEADER, , STENCILA_ARRAY_DIMENSIONS)
			#undef STENCILA_ARRAY_HEADER
			stream<<"value"<<std::endl;
			// Values
			for(uint index=0; index<size(); index++){
				#define STENCILA_ARRAY_ROW(r,data,elem) if(elem::size_>1) stream<<level(elem(),index)<<"\t";
				BOOST_PP_SEQ_FOR_EACH(STENCILA_ARRAY_ROW, , STENCILA_ARRAY_DIMENSIONS)
				#undef STENCILA_ARRAY_ROW
				stream<<operator[](index)<<std::endl;
			}
		}
		else if(format=="bin"){
			static_assert(true,"Not implemented");
		}
		else{
			STENCILA_THROW(Exception,"Unsupported format:"+format)
		}
	}

	/**
	 * Write array to a file
	 * 
	 * @param path Filesystem path to file
	 */
	void write(const std::string& path) const {
		std::string extension = boost::filesystem::extension(path);
		std::ofstream file(path);
		write(file,extension);
		file.close();
	}

};


/**
 * A dynamic array
 *
 * This implementation of array is useful for arrays of variable size.
 * It is a wrapper arounf the C++ std::vector class but has an interface that\
 * is consistent as possible with static Array classes.
 */
template<
	typename Type
>
class Array<Type> {
private:

	std::vector<Type> values_;

public:
    
   	Array(void){
	}

	template<class Other>
    Array(const Other& other){
    	construct_(IsContainer<Other>(),other);
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
        // Convert to size
        unsigned int num = other;
        size(num);
    }

 	template<class Other>
    construct_(const std::true_type& is_container,const Other& other){
        size(other.size());
        unsigned int index = 0;
        for(auto& item : other){
            values_[index] = item;
            index++;
        }
    }

public:
    
	/**
	 * Get the size of the array.
	 */
    unsigned int size(void) const {
        return values_.size();
    }
    
	/**
	 * Set the size of the array.
	 */
    Array size(unsigned int size) {
        values_.resize(size);
        return *this;
    }
    
 	/**
	 * @name Iterator interface
	 *
	 * @{
	 */

   	typename std::vector<Type>::iterator begin(void) {
		return values_.begin();
	}

	typename std::vector<Type>::const_iterator begin(void) const {
		return values_.begin();
	}

	typename std::vector<Type>::iterator end(void) {
		return values_.end();
	}

	typename std::vector<Type>::const_iterator end(void) const {
		return values_.end();
	}    

    /**
     * @}
     */
    

	/**
	 * @name Subscript operators
	 *
	 * Return the value at the linear index
	 * 
	 * @{
	 */

    Type& operator[](unsigned int index) {
        return values_[index];
    }

    const Type& operator[](unsigned int index) const {
        return values_[index];
    }

    /**
     * @}
     */
    
   	/**
   	 * Modification methods
   	 */

    /**
     * Append a value to the array
     * 
     * @param value Value to append
     */
    void append(const Type& value) {
        return values_.push_back(value);
    }

    /**
     * Remove all items equal to a particular value
     * 
     * @param value Value ot be removed
     */
    void remove(const Type& value){
    	values_.erase(std::remove(values_.begin(), values_.end(), value), values_.end());
    }

    /**
     * Erase the element at a particular position
     */
    void erase(unsigned int index) {
        return values_.erase(values_.begin()+index);
    }

    /**
     * @}
     */
    
    #undef STENCILA_ARRAY_DIMENSIONS
};

}
