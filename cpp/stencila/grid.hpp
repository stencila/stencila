#pragma once

#include <fstream>

#include <boost/preprocessor/seq/for_each.hpp>

#include <stencila/array.hpp>
#include <stencila/dimension.hpp>
#include <stencila/exception.hpp>
#include <stencila/query.hpp>
#include <stencila/traits.hpp>

namespace Stencila {

/**
 * A cell of an grid.
 * 
 * Implements an iterator interface for convenient looping
 * over cells in a grid
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
 * A static array
 *
 * An array with fixed, known dimensions
 */
template<
	typename Type = double,
	class D1 = Singular1,
	class D2 = Singular2,
	class D3 = Singular3,
	class D4 = Singular4,
	class D5 = Singular5,
	class D6 = Singular6,
	class D7 = Singular7,
	class D8 = Singular8,
	class D9 = Singular9,
	class D10 = Singular10
>
class Grid {
private:

	// A sequence of dimension numbers used below for application
	// of [BOOST_PP_SEQ_FOR_EACH](http://www.boost.org/doc/libs/1_55_0/libs/preprocessor/doc/ref/seq_for_each.html)
	#define STENCILA_GRID_DIMENSIONS (D1)(D2)(D3)(D4)(D5)(D6)(D7)(D8)(D9)(D10)

	static const unsigned int size_ =
		D1::size_ * D2::size_ * D3::size_ * D4::size_ * D5::size_ *
		D6::size_ * D7::size_ * D8::size_ * D9::size_ * D10::size_;

	Type values_[size_];

	// A templated struct used in method overloading to signify alternative numbers (e.g dimensions; function arity)
	template<unsigned int> struct Rank {};

public:

	/**
	 * @namespace Constructors
	 * @{
	 */

	Grid(void){
	}

	Grid(const Type& other){
		for(Type& value : values_) value = other;
	}

	template<typename Other>
    Grid(const Other& other){
    	construct_dispatch_(IsContainer<Other>(),IsCallable<Other>(),other);
    }

	/**
	 * Construct from an initializer_list (e.g. `{1.23,3.14,5.98}`)
	 *
	 * This constructor appears to be nessary because compiler (gcc 4.81 at least)
	 * can not resolve between above consturtors when called with an intiializer list
	 */
    template<typename Value>
	Grid(const std::initializer_list<Value>& values){
        construct_container_(values);
    }

private:

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
     * Implicit conversion to a std::vector
     */
    operator std::vector<Type>(void) {
        return std::vector<Type>(values_,values_+size_);
    }

	/**
	 * Get the size of the grid.
	 */
    static unsigned int size(void) {
		return size_;
	}

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
	 * Does the grid have a dimension?
	 */
	template<class Dimension>
	static bool dimensioned(const Dimension&) {
		return false;
	}

	// The following macro and BOOST_PP_SEQ_FOR_EACH call create a dimensioned method
	// for each possible dimension
	#define STENCILA_GRID_DIMENSIONED(r,data,elem) static bool dimensioned(const elem&) { return true; }
	BOOST_PP_SEQ_FOR_EACH(STENCILA_GRID_DIMENSIONED, , STENCILA_GRID_DIMENSIONS)
	#undef STENCILA_GRID_DIMENSIONED

	/**
	 * Get the number of cells in a single level of a dimension
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
	 */
	template<class Dimension>
	static uint jump(const Level<Dimension>& level){
		return level.index() * base(Dimension());
	}

	/**
	 * Get the level of a dimension at an index of this grid
	 * 
	 * @param  dimension  The dimension
	 * @param  index The linear index
	 */
	template<class Dimension>
	static Level<Dimension> level(const Dimension& dimension, unsigned int index) {
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
	 * Get the linear index corresponding to particular levels of each 
	 * of the array's dimensions
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

    template<
		class Class, typename Result
	>
	Result operator()(Aggregate<Class,Result>& aggregate) const{
		for(auto& value : *this) aggregate.append(value);
		return aggregate.result();
	}
	
	template<
		class Class, typename Result,
		class A1,class A2,class A3,class A4,class A5,class A6,class A7,class A8,class A9,class A10
	>
	Grid<Result,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10> operator()(const Aggregate<Class,Result>& aggregate,const By<A1,A2,A3,A4,A5,A6,A7,A8,A9,A10>& by) const{
		Grid<Class,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10> aggregates;
		for(uint index=0;index<size();index++) {
			aggregates(
				level(A1(),index),Level<A2>(level(A2(),index)),level(A3(),index),level(A4(),index),level(A5(),index),
				level(A6(),index),level(A7(),index),level(A8(),index),level(A9(),index),level(A10(),index)
			).append(operator[](index));
		}
		Grid<Result,A1,A2,A3,A4,A5,A6,A7,A8,A9,A10> results;
		for(int index=0;index<aggregates.size();index++) results[index] = aggregates[index].result();
		return results;
	}

	Array<> operator()(const Query& query) const {
		for(Clause* clause : query){
			if(Counter* counter = dynamic_cast<Counter*>(clause)){
				for(auto& value : *this) counter->append();
				return {counter->result()};
			} else if(Aggregater<double,double>* aggregater = dynamic_cast<Aggregater<double,double>*>(clause)){
				for(auto& value : *this) aggregater->append(value);
				return {aggregater->result()};
			} else {
				STENCILA_THROW(Exception,"Query clause can not be applied");
			}
		}
		return Array<>();
	}

	/**
	 * @}
	 */
	

	/**
	 * Numeric operator overloading
	 */
	
	#define STENCILA_LOCAL(op) \
		template<class Value> \
		Grid& operator op (const Value& value) { \
			for(auto& cell : *this) cell op value; \
			return *this; \
		}
	STENCILA_LOCAL(+=)
	STENCILA_LOCAL(-=)
	STENCILA_LOCAL(*=)
	STENCILA_LOCAL(/=)
	#undef STENCILA_LOCAL
	
	void read(std::istream& stream,void(*function)(std::istream&,Type&)){
		// Read in the header
		// Currently this is not checked for consistency with the grid dimension names
		std::string header;
		std::getline(stream,header);

		std::string line;
		while(std::getline(stream,line)){
			// Check for lines that are all whitespace and skip them
			// (this primarily is to prevent errors caused by extra empty lines at end of files)
			if(std::all_of(line.begin(),line.end(),isspace)) continue;
			// Put line into a string stream for reading
			std::stringstream line_stream(line);
			uint index = 0;
			Type value;
			try{
				// Accumulate index
				#define STENCILA_LOCAL(r,data,dimension) if(dimension::size_>1) index += jump(dimension::level(line_stream));
				BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, ,STENCILA_GRID_DIMENSIONS)
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

	void read(const std::string& path,void(*function)(std::istream&,Type&)){
		std::ifstream file(path);
		read(file,function);
		file.close();
	}

	void write(std::ostream& stream, const std::vector<std::string>& names, void(*function)(std::ostream&,const Type&)) const {
		// Header
		#define STENCILA_GRID_HEADER(r,data,dimension) if(dimension::size_>1) stream<<dimension::name()<<"\t";
		BOOST_PP_SEQ_FOR_EACH(STENCILA_GRID_HEADER, , STENCILA_GRID_DIMENSIONS)
		#undef STENCILA_GRID_HEADER
		// Add header names for function
		for(auto& name : names) stream<<name<<"\t";
		// End the line
		stream<<std::endl;

		// Values
		for(uint index=0;index<size();index++){

			#define STENCILA_GRID_ROW(r,data,dimension) if(dimension::size_>1) stream<<level(dimension(),index).label()<<"\t";
			BOOST_PP_SEQ_FOR_EACH(STENCILA_GRID_ROW, , STENCILA_GRID_DIMENSIONS)
			#undef STENCILA_GRID_ROW

			function(stream,values_[index]);

			stream<<std::endl;
		}
	}

	void write(const std::string& path, const std::vector<std::string>& names, void(*function)(std::ostream&,const Type&)) const {
		std::ofstream file(path);
		write(file,names,function);
		file.close();
	}

	/**
	 * Write array to an output stream
	 * 
	 * @param stream Output stream
	 * @param format Format specifier string (e.g. "tsv", "csv")
	 *
	 * @todo Implement more output formats including tuning off header and binary output
	 */
	void write(std::ostream& stream,const std::string format) const {
		if(format=="tsv"){
			// Header
			
			#define STENCILA_GRID_HEADER(r,data,dimension) if(dimension::size_>1) stream<<dimension::name()<<"\t";
			BOOST_PP_SEQ_FOR_EACH(STENCILA_GRID_HEADER, , STENCILA_GRID_DIMENSIONS)
			#undef STENCILA_GRID_HEADER

			stream<<"value"<<std::endl;
			// Values
			for(uint index=0;index<size();index++){

				#define STENCILA_GRID_ROW(r,data,dimension) if(dimension::size_>1) stream<<level(dimension(),index).label()<<"\t";
				BOOST_PP_SEQ_FOR_EACH(STENCILA_GRID_ROW, , STENCILA_GRID_DIMENSIONS)
				#undef STENCILA_GRID_ROW

				stream<<values_[index];

				stream<<std::endl;
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
		std::string extension = boost::filesystem::extension(path).substr(1);
		std::ofstream file(path);
		write(file,extension);
		file.close();
	}

#undef STENCILA_GRID_DIMENSIONS

};


/**
 * Output a static array to a stream using the `<<` operator
 */
template<
	class Type,
	class... Dimensions
>
std::ostream& operator<<(std::ostream& stream, const Grid<Type,Dimensions...>& array){
	array.write(stream);
	return stream;
}

} //namespace Stencila
