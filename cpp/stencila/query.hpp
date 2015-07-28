#pragma once

#include <cmath>

#include <stencila/polymorph.hpp>
#include <stencila/traits.hpp>
#include <stencila/dimension.hpp>

namespace Stencila {

namespace Queries {}

/**
 * An element of a Query
 */
class Clause {
public:

	/**
	 * Get the code representation of the clause
	 */
	virtual std::string code(void) const {
		return "";
	};

};


template<
	typename Values,
	typename Result
>
class AggregateDynamic : public Clause {
public:

	virtual void append_dynamic(const Values& value) = 0;
	virtual Result result_dynamic(void) const = 0;

};

template<
	class Derived,
	typename Values,
	typename Result
>
class Aggregate : public AggregateDynamic<Values,Result>, public Polymorph<Derived> {
 public:
 	using Polymorph<Derived>::derived;
	typedef Result result_type;

	/**
	 * Apply the aggregator to an object
	 */
	template<typename Type>
	Derived& apply(const Type& object) {
		derived().reset();
		append(object);
		return derived();
	}

	template<typename Type,typename Member>
	Derived& apply(const Type& object, Member member) {
		derived().reset();
		append(object, member);
		return derived();
	}

	/**
	 * Append a container, array or value
	 */
	template<typename Type>
	Derived& append(const Type& object){
		append_(object,IsContainer<Type>(),IsArray<Type>());
		return derived();
	}

	template<typename Type,typename Member>
	Derived& append(const Type& object, Member member){
		append_member_(object,member,std::true_type(),typename std::is_member_function_pointer<Member>::type());
		return derived();
	}

	/**
	 * Append an item dynamically
	 */
	void append_dynamic(const Values& value){
		append(value);
	}

	/**
	 * Dump this aggregator to a string.
	 * Used to store, and then later combine, aggregators.
	 *
	 * Should be overidden by derived classes.
	 */
	std::string dump(void){
		return "";
	}
	
	/**
	 * Load this aggregator from a string.
	 * Used to load a stored aggregator.
	 *
	 * Should be overidden by derived classes.
	 * 
	 * @param  value String codeesentation
	 */
	Derived& load(const std::string& value){
		return derived();
	}
	
	/**
	 * Join two aggregators of the same class.
	 * Used to join aggregator instances that have been run
	 * on different database table shards or segments of arrays.
	 *
	 * Should be overidden by derived classes.
	 * 
	 * @param  other Other aggregator instance
	 */
	Derived& join(const Derived& other){
		return derived();
	}

	/**
	 * Get the result of the aggregator
	 */
	Result result(void) const {
		return derived().result_static();
	}

	/**
	 * Get the result of the aggregator dynamically
	 */
	Result result_dynamic(void) const {
		return result();
	}

	/**
	 * Implicit conversion to result type by
	 * caling `calc()`
	 */
	operator Result(void) const {
		return result();
	}

private:

	template<typename Type>
	void append_(Type container, const std::true_type& is_container,const std::false_type& is_array) {
		for(auto& value : container) derived().append_static(value);
	}

	template<typename Type>
	void append_(Type array, const std::false_type& is_container,const std::true_type& is_array) {
		for(auto& value : array) derived().append_static(value);
	}

	template<typename Type>
	void append_(const Type& value, const std::false_type& is_container,const std::false_type& is_array){
		derived().append_static(value);
	}

	template<typename Type,typename Member>
	void append_member_(Type container, Member member, const std::true_type& is_container,const std::true_type& is_method){
		for(auto& item : container){
			auto value = (item.*member)();
			derived().append_static(value);
		}
	}

	template<typename Type,typename Member>
	void append_member_(Type container, Member member, const std::true_type& is_container,const std::false_type& is_method){
		for(auto& item : container){
			auto value = item.*member;
			derived().append_static(value);
		}
	}
};

template<
	typename Type,
	typename Function
>
class Each : public Aggregate<Each<Type,Function>,Type,void> {
private:

	Function function_;

public:

	virtual std::string code(void) const{
		return "each";
	}

	Each(Function function):
		function_(function){}

	void reset(void){
	}

	Each& append_static(const Type& value){
		function_(value);
		return *this;
	}

	void result_static(void) const {
	}

};

template<class Type,typename Container,typename Function>
void each(const Container& container, Function function){
	return Each<Type,Function>(function).apply(container).result();
}

namespace Queries { using Stencila::each; }


class Count : public Aggregate<Count,double,unsigned int> {
protected:

	double count_;
	
public:
	Count(void):
		count_(0){
	}

	void reset(void){
		count_ = 0;
	}

	virtual std::string code(void) const{
		return "count";
	}

	using Aggregate<Count,double,unsigned int>::append;
	
	Count& append(void){
		count_++;
		return *this;
	}

	template<class Type>
	void append_static(const Type& value){
		count_++;
	}

	std::string dump(void) const {
		char value[1000];
		std::sprintf(value, "%lf", count_);
		return value;
	}
	
	Count& load(const std::string& value){
		std::sscanf(value.c_str(), "%lf", &count_);
		return *this;
	}
	
	Count& join(const Count& other){
		count_ += other.count_;
		return *this;
	}

	double result_static(void) const {
		return count_;
	}

};

static Count count(void){
	return Count();
}

template<typename... Args>
Count count(Args... args){
	Count count;
	count.apply(args...);
	return count;
}

namespace Queries {
	using Stencila::count;
	using Stencila::Count;
}


class Frequency : public Aggregate<Frequency,unsigned int,std::vector<unsigned int>> {
protected:

	std::vector<unsigned int> counts_;
	
public:
	Frequency(void):
		counts_(0){
	}

	void reset(unsigned int size){
		counts_.clear();
		counts_.resize(size);
	}

	virtual std::string code(void) const{
		return "freq";
	}

	template<class Type>
	void append_static(const Type& value){
		unsigned int index = value;
		if(counts_.size()>index) counts_[index]++;
		else {
			counts_.resize(index+1);
			counts_[index] = 1;
		}
	}

	result_type result_static(void) const {
		return counts_;
	}

};

namespace Queries {
	using Stencila::Frequency;
}




class Sum : public Aggregate<Sum,double,double> {
protected:

	double sum_;
	
public:
	Sum(void):
		sum_(0){
	}

	void reset(void){
		sum_ = 0;
	}

	virtual std::string code(void) const {
		return "sum";
	}

	template<class Type>
	void append_static(const Type& value){
		sum_ += value;
	}

	std::string dump(void) const {
		char value[1000];
		std::sprintf(value, "%lf", sum_);
		return value;
	}

	Sum& load(const std::string& value){
		std::sscanf(value.c_str(), "%lf", &sum_);
		return *this;
	}
	
	Sum& join(const Sum& other){
		sum_ += other.sum_;
		return *this;
	}

	double result_static(void) const {
		return sum_;
	}
};

template<typename... Args>
Sum sum(Args... args){
	Sum sum;
	sum.apply(args...);
	return sum;
}

static Sum sum(void){
	return Sum();
}

namespace Queries {
	using Stencila::sum;
	using Stencila::Sum;
}


class Product : public Aggregate<Product,double,double> {
protected:
	double prod_;
	
public:
	Product(void):
		prod_(1){
	}

	void reset(void){
		prod_ = 1;
	}

	virtual std::string code(void) const{
		return "prod";
	}

	template<class Type>
	void append_static(const Type& value){
		prod_ *= value;
	}

	std::string dump(void) const {
		char value[1000];
		std::sprintf(value, "%lf", prod_);
		return value;
	}

	Product& load(const std::string& value){
		std::sscanf(value.c_str(), "%lf", &prod_);
		return *this;
	}
	
	Product& join(const Product& other){
		prod_ *= other.prod_;
		return *this;
	}

	double result_static(void) const {
		return prod_;
	}

};
static Product prod;

namespace Queries {
	using Stencila::Product;
	using Stencila::prod;
}

class Mean : public Aggregate<Mean,double,double> {
private:
	double sum_;
	double count_;

public:
	Mean(void):
		sum_(0),count_(0){
	}

	void reset(void){
		sum_ = 0;
		count_ = 0;
	}

	virtual std::string code(void) const{
		return "mean";
	}

	template<class Type>
	void append_static(const Type& value){
		sum_ += value;
		count_++;
	}

	std::string dump(void) const {
		char value[1000];
		std::sprintf(value, "%lf %lf", sum_, count_);
		return value;
	}

	Mean& load(const std::string& value){
		std::sscanf(value.c_str(), "%lf %lf", &sum_, &count_);
		return *this;
	}
	
	Mean& join(const Mean& other){
		sum_ += other.sum_;
		count_ += other.count_;
		return *this;
	}

	double result_static(void) const {
		return sum_/count_;
	}
};
static Mean mean;

namespace Queries {
	using Stencila::Mean;
	using Stencila::mean;
}

class GeometricMean : public Aggregate<GeometricMean,double,double> {
private:

	Mean mean_;

public:

	void reset(void){
		mean_.reset();
	}

	virtual std::string code(void) const{
		return "geomean";
	}

	template<class Type>
	void append_static(const Type& value){
		if(value>0) mean_.append(std::log(value));
	}

	std::string dump(void) const {
		return mean_.dump();
	}

	GeometricMean& load(const std::string& value){
		mean_.load(value);
		return *this;
	}
	
	GeometricMean& join(const GeometricMean& other){
		mean_.join(other.mean_);
		return *this;
	}

	double result_static(void) const {
		return std::exp(mean_.result());
	}
};
static GeometricMean geomean;

namespace Queries {
	using Stencila::GeometricMean;
	using Stencila::geomean;
}

class HarmonicMean : public Aggregate<GeometricMean,double,double> {
private:

	Mean mean_;

public:

	void reset(void){
		mean_.reset();
	}

	virtual std::string code(void) const{
		return "harmean";
	}

	template<class Type>
	void append_static(const Type& value){
		if(value!=0) mean_.append_static(1.0/value);
	}

	std::string dump(void) const {
		return mean_.dump();
	}

	HarmonicMean& load(const std::string& value){
		mean_.load(value);
		return *this;
	}
	
	HarmonicMean& join(const HarmonicMean& other){
		mean_.join(other.mean_);
		return *this;
	}

	double result_static(void) const {
		return 1.0/mean_.result_static();
	}
};
static HarmonicMean harmean;

namespace Queries {
	using Stencila::HarmonicMean;
	using Stencila::harmean;
}


class Variance : public Aggregate<Variance,double,double> {
public:
	Variance(void):
		count_(0),
		mean_(0),
		m2_(0){
	}

	void reset(void){
		count_ = 0;
		mean_ = 0;
		m2_ = 0;
	}

	void append_static(const double& value){
		count_++;
		double delta = value - mean_;
		mean_ += delta/count_;
		m2_ += delta*(value-mean_);
	}

	std::string dump(void){
		char value[1000];
		std::sprintf(value, "%li %lf %lf", count_, mean_, m2_);
		return value;
	}
	
	void load(const std::string& value){
		std::sscanf(value.c_str(), "%li %lf %lf", &count_, &mean_, &m2_);
	}
	
	void join(const Variance& other){
		count_ += other.count_;
		mean_ += other.mean_;
		m2_ += other.m2_;
	}
	 
	double result_static(void) const {
		return m2_/(count_ - 1);
	}

protected:
	unsigned long int count_;
	double mean_;
	double m2_;
};

class StandardDeviation : public Variance {
public:
	double result_static(void) const {
		return std::sqrt(Variance::result_static());
	}
};

class Mapc : public Aggregate<Mapc,double,double> {
private:

	Mean mean_;
	double last_ = NAN;

public:

	void reset(void){
		mean_.reset();
		last_ = NAN;
	}

	virtual std::string code(void) const{
		return "mapc";
	}

	template<class Type>
	void append_static(const Type& value){
		if(std::isfinite(last_)){
			mean_.append_static(std::fabs(value-last_)/last_);
		}
		last_ = value;
	}

	std::string dump(void) const {
		return mean_.dump();
	}

	Mapc& load(const std::string& value){
		mean_.load(value);
		return *this;
	}
	
	Mapc& join(const Mapc& other){
		mean_.join(other.mean_);
		return *this;
	}

	double result_static(void) const {
		return mean_.result_static();
	}
};
static Mapc mapc;


/**
 * `by` query specialised for `Array`s.
 *
 * This `By` class contains dimensions which can
 * be used by `Array` class to optimise it's running.
 */
template<
	class D1,
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
class By {
public:
};

template<class... Dimension>
By<Dimension...> by(Dimension... dims){
	return By<Dimension...>();
}


class Query : public std::vector<Clause*> {
public:

	Query(void){
	}

	/**
	 * Construct a query from a single `Clause`
	 */
	Query(Clause* clause){
		push_back(clause);
	}
};


}
