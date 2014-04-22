#pragma once

#include <fstream>

#include <boost/preprocessor/seq/for_each.hpp>

#include <stencila/array-declaration.hpp>
#include <stencila/exception.hpp>
#include <stencila/query.hpp>
#include <stencila/traits.hpp>

namespace Stencila {

/**
 * A cell of an array.
 * 
 * Implements an iterator interface for convenient looping
 * over cells in an array
 */
template<class Type>
class Cell {
private:
	Type* value_;

public:
	Cell(Type* value):
		value_(value){
	}

	/**
	 * Dereference.
	 */
	Type& operator*() const { 
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


/**
 * @name Array
 * 
 * A multi-dimensional data structure
 */
template<
	typename Type,
	class D1,
	class D2,
	class D3,
	class D4,
	class D5,
	class D6,
	class D7,
	class D8,
	class D9,
	class D10
>
class Array {
private:

	// A sequence of dimension numbers used below for application
	// of [BOOST_PP_SEQ_FOR_EACH](http://www.boost.org/doc/libs/1_55_0/libs/preprocessor/doc/ref/seq_for_each.html)
	#define STENCILA_array_DIMENSIONS (D1)(D2)(D3)(D4)(D5)(D6)(D7)(D8)(D9)(D10)

	/**
	 * Size of the array, a product of the size of each dimension
	 */
	static const unsigned int size_ =
		D1::size_ * D2::size_ * D3::size_ * D4::size_ * D5::size_ *
		D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;

	/**
	 * Stored values
	 */
	Type values_[size_];

	// A templated struct used in method overloading to signify alternative numbers (e.g dimensions; function arity)
	template<unsigned int> struct Rank {};

public:

	/**
	 * @namespace Constructors
	 * @{
	 */

	/**
	 * Default constructor
	 */
	Array(void){
	}

	/**
	 * Construct from another array with same dimensions
	 */
	Array(const Type& other){
		for(Type& value : values_) value = other;
	}

	/**
	 * Construct from some other object (e.g function, atomic).
	 * See the constructor helper group of methods below
	 */
	template<typename Other>
    Array(const Other& other){
    	construct_dispatch_(IsContainer<Other>(),IsCallable<Other>(),other);
    }

	/**
	 * Construct from an initializer_list (e.g. `{1.23,3.14,5.98}`)
	 *
	 * This constructor appears to be nessary because compiler (gcc 4.81 at least)
	 * can not resolve between above consturtors when called with an initializer list
	 */
    template<typename Value>
	Array(const std::initializer_list<Value>& values){
        construct_container_(values);
    }

private:

	/**
	 * @name Constructor helpers
	 *
	 * A group of methods for helping to construct Array from various types of objects
	 * 
	 * @{
	 */
 	template<typename Other>
    void construct_dispatch_(const std::false_type& is_container,const std::false_type& is_callable,const Other& other){
        construct_atomic_(other);
    }

 	template<typename Other>
    void construct_dispatch_(const std::false_type& is_container,const std::true_type& is_callable,const Other& other){
        construct_callable_(other);
    }

 	template<typename IsCallable, typename Other>
    void construct_dispatch_(const std::true_type& is_container,const IsCallable& is_callable,const Other& other){
        construct_container_(other);
    }

 	template<typename Atomic>
    void construct_atomic_(const Atomic& atomic){
        for(Type& value : values_) value = atomic;
    }

 	template<class Container>
    void construct_container_(const Container& container){
        uint index = 0;
        for(auto& item : container){
            values_[index] = item;
            index++;
            if(index>=size_) break;
        }
    }

	template<typename Callable>
    void construct_callable_(Callable callable){
    	typedef FunctionTraits<decltype(callable)> traits;
    	for(unsigned int index=0;index<size();index++) values_[index] = construct_call_(Rank<traits::arity>(),index,callable);
	}

	template<typename Callable> static Type construct_call_(Rank<0>,unsigned int index,Callable callable){
		return callable();
	}
	template<typename Callable>	static Type construct_call_(Rank<1>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<2>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<3>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<4>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<5>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index),
			level(D5(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<6>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index),
			level(D5(),index),
			level(D6(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<7>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index),
			level(D5(),index),
			level(D6(),index),
			level(D7(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<8>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index),
			level(D5(),index),
			level(D6(),index),
			level(D7(),index),
			level(D8(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<9>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index),
			level(D5(),index),
			level(D6(),index),
			level(D7(),index),
			level(D8(),index),
			level(D9(),index)
		);
	}
	template<typename Callable>	static Type construct_call_(Rank<10>,unsigned int index,Callable callable){
		return callable(
			level(D1(),index),
			level(D2(),index),
			level(D3(),index),
			level(D4(),index),
			level(D5(),index),
			level(D6(),index),
			level(D7(),index),
			level(D8(),index),
			level(D9(),index),
			level(D10(),index)
		);
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
	 * Does the array have a dimension?
	 */
	template<class Dimension>
	static bool dimensioned(const Dimension&) {
		return false;
	}

	// The following macro and BOOST_PP_SEQ_FOR_EACH call create a dimensioned method
	// for each possible dimension
	#define STENCILA_LOCAL(r,data,elem) static bool dimensioned(const elem&) { return true; }
	BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, , STENCILA_array_DIMENSIONS)
	#undef STENCILA_LOCAL

 	/**
	 * @name Iterator interface
	 *
	 * @{
	 */

	Cell<const Type> begin(void) const {
		return Cell<const Type>(&values_[0]);
	}

	Cell<const Type> end(void) const {
		return Cell<const Type>(&values_[size_]);
	}    

	Cell<Type> begin(void) {
		return Cell<Type>(&values_[0]);
	}

	Cell<Type> end(void) {
		return Cell<Type>(&values_[size_]);
	}    

    /**
     * @}
     */
    
	/**
	* Implicit conversion to a std::vector
	*/
    operator std::vector<Type>(void) {
        return std::vector<Type>(values_,values_+size_);
    }

	/**
	 * Get the number of cells in a single level of a dimension.
	 *
	 * This method is used by the jump and level methods below.
	 */
	template<class Dimension>
	static uint base(const Dimension&) { 
		return 0;
	}
	static uint base(const D1&) { 
		return D2::size_ * D3::size_ * D4::size_ * D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D2&) { 
		return D3::size_ * D4::size_ * D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D3&) { 
		return D4::size_ * D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D4&) { 
		return D5::size_ * D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D5&) { 
		return D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D6&) { 
		return D7::size_ * D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D7&) { 
		return D8::size_ * D9::size_ * D10::size_;
	}
	static uint base(const D8&) { 
		return D9::size_ * D10::size_;
	}
	static uint base(const D9&) { 
		return D10::size_;
	}
	static uint base(const D10&) { 
		return 1;
	}

	/**
	 * Get the jump in the index associated with a level of a dimension
	 *
	 * @param level Level of the dimension
	 */
	template<class Dimension>
	static uint jump(const Level<Dimension>& level){
		return level.index() * base(Dimension());
	}

	/**
	 * Get the level of a dimension at an index of this array
	 * 
	 * @param  dimension  Dimension for which level is to be obtained
	 * @param  index 	  Index of this array to be translated into a level of the dimension
	 */
	template<class Dimension>
	static Level<Dimension> level(const Dimension& dimension, unsigned int index) {
		// This method is called when this array does not contain the dimension.
		// It returns a "null" Level<Dimension> which has an index of zero. 
		return dimension.level();
	}
	static Level<D1> level(const D1& dimension, unsigned int index) {
		return Level<D1>(index/base(dimension),"index");
	}
	static Level<D2> level(const D2& dimension, unsigned int index) {
		return Level<D2>(index/base(dimension)%D2::size_,"index");
	}
	static Level<D3> level(const D3& dimension, unsigned int index) {
		return Level<D3>(index/base(dimension)%D3::size_,"index");
	}
	static Level<D4> level(const D4& dimension, unsigned int index) {
		return Level<D4>(index/base(dimension)%D4::size_,"index");
	}
	static Level<D5> level(const D5& dimension, unsigned int index) {
		return Level<D5>(index/base(dimension)%D5::size_,"index");
	}
	static Level<D6> level(const D6& dimension, unsigned int index) {
		return Level<D6>(index/base(dimension)%D6::size_,"index");
	}
	static Level<D7> level(const D7& dimension, unsigned int index) {
		return Level<D7>(index/base(dimension)%D7::size_,"index");
	}
	static Level<D8> level(const D8& dimension, unsigned int index) {
		return Level<D8>(index/base(dimension)%D8::size_,"index");
	}
	static Level<D9> level(const D9& dimension, unsigned int index) {
		return Level<D9>(index/base(dimension)%D9::size_,"index");
	}
	static Level<D10> level(const D10& dimension, unsigned int index) {
		return Level<D10>(index/base(dimension)%D10::size_,"index");
	}

	/**
	 * Get the index of this array corresponding to particular levels of each 
	 * of the it's dimensions
	 */
	static unsigned int index(
		const Level<D1>& level1,
		const Level<D2>& level2 = Level<Singular2>(0),
		const Level<D3>& level3 = Level<Singular3>(0),
		const Level<D4>& level4 = Level<Singular4>(0),
		const Level<D5>& level5 = Level<Singular5>(0),
		const Level<D6>& level6 = Level<Singular6>(0),
		const Level<D7>& level7 = Level<Singular7>(0),
		const Level<D8>& level8 = Level<Singular8>(0),
		const Level<D9>& level9 = Level<Singular9>(0),
		const Level<D10>& level10 = Level<Singular10>(0)
	) {
		return 
			jump(level1) + jump(level2) + jump(level3) + jump(level4) + jump(level5) + 
			jump(level6) + jump(level7) + jump(level8) + jump(level9) + jump(level10)
		;
	}

	/**
	 * @name Subscript operators
	 *
	 * Return the value at the index
	 * 
	 * @{
	 */

	Type& operator[](uint index){
		return values_[index];
	}

	const Type& operator[](uint index) const {
		return values_[index];
	}

	Type& operator()(
		const Level<D1>& l1,
		const Level<D2>& l2 = Level<Singular2>(0),
		const Level<D3>& l3 = Level<Singular3>(0),
		const Level<D4>& l4 = Level<Singular4>(0),
		const Level<D5>& l5 = Level<Singular5>(0),
		const Level<D6>& l6 = Level<Singular6>(0),
		const Level<D7>& l7 = Level<Singular7>(0),
		const Level<D8>& l8 = Level<Singular8>(0),
		const Level<D9>& l9 = Level<Singular9>(0),
		const Level<D10>& l10 = Level<Singular10>(0)
	){
		return values_[index(l1,l2,l3,l4,l5,l6,l7,l8,l9,l10)];
	}

	const Type& operator() (
		const Level<D1>& l1,
		const Level<D2>& l2 = Level<Singular2>(0),
		const Level<D3>& l3 = Level<Singular3>(0),
		const Level<D4>& l4 = Level<Singular4>(0),
		const Level<D5>& l5 = Level<Singular5>(0),
		const Level<D6>& l6 = Level<Singular6>(0),
		const Level<D7>& l7 = Level<Singular7>(0),
		const Level<D8>& l8 = Level<Singular8>(0),
		const Level<D9>& l9 = Level<Singular9>(0),
		const Level<D10>& l10 = Level<Singular10>(0)
	) const {
		return values_[index(l1,l2,l3,l4,l5,l6,l7,l8,l9,l10)];
	}

	/**
	 * @}
	 */

	/**
	 * @name Query operators
	 *
	 * Evaluate a query on this array
	 * 
	 * @{
	 */
	
	/**
	 * Evaluate a dynamic query and return an array with the results.
	 *
	 * Currently, this is a partial implementation which does not handle all query types.
	 */
	Array<> operator()(const Query& query) const {
		for(Clause* clause : query){
            if(AggregateDynamic<double,uint>* aggregate = dynamic_cast<AggregateDynamic<double,uint>*>(clause)){
                for(auto& value : *this) aggregate->append_dynamic(value);
                return {aggregate->result_dynamic()};
            }
            else if(AggregateDynamic<double,double>* aggregate = dynamic_cast<AggregateDynamic<double,double>*>(clause)){
                for(auto& value : *this) aggregate->append_dynamic(value);
                return {aggregate->result_dynamic()};
            }
            else {
                STENCILA_THROW(Exception,"Query clause can not be applied: "+clause->code());
            }
		}
		return Array<>();
	}

	/**
	 * Evaluate an `Aggregate` type query and return its result
	 */
    template<
		class Derived, typename Values, typename Result
	>
	Result operator()(Aggregate<Derived,Values,Result>& aggregate) const{
		for(auto& value : *this) aggregate.append(value);
		return aggregate.result();
	}
	
	/**
	 * Evaluate an `Aggregate` and `By` query combination returning
	 * a `Array` with the same dimensions as the `By`.
	 */
	template<
		class Derived, typename Values, typename Result,
		class A1,class A2,class A3,class A4,class A5,class A6,class A7,class A8,class A9,class A10
	>
	Array<Result,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10> operator()(const Aggregate<Derived,Values,Result>& aggregate,const By<A1,A2,A3,A4,A5,A6,A7,A8,A9,A10>& by) const{
		// Create an array of aggregators with the dimesnions of the Byer
		Array<Derived,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10> aggregates;
		// Iterate over tthis array updating the appropriate level of the aggregators array
		for(uint index=0;index<size();index++) {
			aggregates(
				level(A1(),index),level(A2(),index),level(A3(),index),level(A4(),index),level(A5(),index),
				level(A6(),index),level(A7(),index),level(A8(),index),level(A9(),index),level(A10(),index)
			).append(operator[](index));
		}
		// Get the results of each aggregator
		Array<Result,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10> results;
		for(int index=0;index<aggregates.size();index++) results[index] = aggregates[index].result();
		return results;
	}

	/**
	 * @}
	 */
	

	/**
	 * @name Numeric operators
	 *
	 * @{
	 */
	
	#define STENCILA_LOCAL(op) \
		template<class Value> \
		Array& operator op (const Value& value) { \
			for(auto& cell : *this) cell op value; \
			return *this; \
		}
	STENCILA_LOCAL(+=)
	STENCILA_LOCAL(-=)
	STENCILA_LOCAL(*=)
	STENCILA_LOCAL(/=)
	#undef STENCILA_LOCAL

	/**
	 * @}
	 */
	

	/**
	 * @name Reading and writing methods
	 * 
	 * @{
	 */

	/**
	 * Read the array from an input stream
	 *
	 * Currently, only tab separated value (TSV) format is supported. Other
	 * formats, including binary, may be implemented in the future.
	 * 
	 * @param stream Input stream
	 * @param function A function that reads tab separated values representing the `Type`
	 */
	void read(std::istream& stream,void(*function)(std::istream&,Type&)){
		// Read in the header
		// Currently this is not checked for consistency with the array dimension names
		std::string header;
		std::getline(stream,header);
		// Get each line....
		std::string line;
		while(std::getline(stream,line)){
			// Check for lines that are all whitespace and skip them
			// (this primarily is to prevent errors caused by extra empty lines at end of files)
			if(std::all_of(line.begin(),line.end(),isspace)) continue;
			// Put line into a string stream for reading by the function
			std::stringstream line_stream(line);
			uint index = 0;
			Type value;
			try{
				// Accumulate index
				#define STENCILA_LOCAL(r,data,dimension) if(dimension::size_>1) index += jump(dimension::level(line_stream));
				BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, ,STENCILA_array_DIMENSIONS)
				#undef STENCILA_LOCAL
				// Read in value using function
				function(line_stream,value);
			} catch(...) {
				STENCILA_THROW(Exception,"Error occurred reading line:"+line);
			}
			// Assign to correct place
			values_[index] = value;
		}
	}

	/**
	 * Read array from an input stream using the >> operator to read each value
	 * 
	 * @param stream Input stream
	 */
	void read(std::istream& stream) {
		read(stream,[](std::istream& stream,Type& value){
			stream>>value;
		});
	}

	/**
	 * Read array from an input file using the specified function to write each value
	 * 
	 * @param path Filesystem path to file
	 * @param function A function that reads tab separated values representing the `Type`
	 */
	void read(const std::string& path, void(*function)(std::istream&,Type&)) {
		std::ifstream file(path);
		read(file,function);
		file.close();
	}

	/**
	 * Read array from an input file using the >> operator to read each value
	 * 
	 * @param path Filesystem path to file
	 */
	void read(const std::string& path) {
		std::ofstream file(path);
		read(file);
		file.close();
	}

	/**
	 * Write the array to an output stream.
	 *
	 * Currently, only tab separated value (TSV) format is supported. Other
	 * formats, including binary, may be implemented in the future.
	 *
	 * @param stream Output stream
	 * @param names Vector of names coresponding to the tab separated values output by `function`
	 * @param function A function that outputs tab separated values representing the `Type`
	 */
	void write(std::ostream& stream, const std::vector<std::string>& names, void(*function)(std::ostream&,const Type&)) const {
		// Write a header row...
		// ...with the names of each of the non-singular dimensions
		#define STENCILA_LOCAL(r,data,dimension) if(dimension::size_>1) stream<<dimension::name()<<"\t";
		BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, ,STENCILA_array_DIMENSIONS)
		#undef STENCILA_LOCAL
		// ...and the names of values that will be output by the function
		uint index = 0;
		for(auto& name : names){
			stream<<name;
			if(index++ < names.size()-1) stream<<"\t";
		}
		// ...then end the header line
		stream<<std::endl;

		// Write value rows...
		for(uint index=0;index<size();index++){
			//...with labels for each nn-singular dimension
			#define STENCILA_LOCAL(r,data,dimension) if(dimension::size_>1) stream<<level(dimension(),index)<<"\t";
			BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, ,STENCILA_array_DIMENSIONS)
			#undef STENCILA_LOCAL
			//...call the function to write the vaue
			function(stream,values_[index]);
			// ...then end the value line
			stream<<std::endl;
		}
	}

	/**
	 * Write array to an output stream using the << operator to write each value
	 * 
	 * @param stream Output stream
	 */
	void write(std::ostream& stream) const {
		write(stream,{"value"},[](std::ostream& stream,const Type& value){
			stream<<value;
		});
	}

	/**
	 * Write array to an output file using the specified function operator to write each value
	 * 
	 * @param path Filesystem path to file
	 * @param names Vector of names coresponding to the tab separated values output by `function`
	 * @param function A function that outputs tab separated values for the Type
	 */
	void write(const std::string& path, const std::vector<std::string>& names, void(*function)(std::ostream&,const Type&)) const {
		std::ofstream file(path);
		write(file,names,function);
		file.close();
	}

	/**
	 * Write array to an output file using the << operator to write each value
	 * 
	 * @param path Filesystem path to file
	 */
	void write(const std::string& path) const {
		std::ofstream file(path);
		write(file);
		file.close();
	}

	/**
	 * @}
	 */
	
#undef STENCILA_array_DIMENSIONS

};

/**
 * Output an array to a stream using the `<<` operator
 */
template<
	class Type,
	class... Dimensions
>
std::ostream& operator<<(std::ostream& stream, const Array<Type,Dimensions...>& array){
	array.write(stream);
	return stream;
}

} //namespace Stencila
