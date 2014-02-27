#pragma once

#include <stencila/traits.hpp>

namespace Stencila {

/**
 * An element of a Query
 */
class Clause {
public:

	/**
	 * Get the programming code representation of the clause
	 */
    virtual std::string code(void) const = 0;

};

class Query : public std::vector<Clause*> {
public:

	Query(void){
	}

	/**
	 * Construct a query from a single clause
	 */
	Query(Clause* clause){
		push_back(clause);
	}
};

template<
	class Class,
	typename Result = double
>
class Aggregator {
public:

	typedef Result result_type;
	
	/**
	 * Convienience method to return address as derived class
	 */
	const Class& self(void) const {
		return static_cast<const Class&>(*this);
	}

	/**
	 * Convienience method to return address as derived class
	 */
	Class& self(void) {
		return static_cast<Class&>(*this);
	}

	/**
	 * Append an item
	 *
	 * Should be overidden by derived classes.
	 */
	template<class Type>
	Class& append(const Type& value){
		return self().add(value);
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
    Class& load(const std::string& value){
        return self();
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
    Class& join(const Class& other){
        return self();
    }

    /**
     * Finalise the aggregator calculations
     *
     * Should be overidden by derived classes.
     */
	Result result(void) const {
		return self().calc();
	}

	/**
	 * Implicit conversion to result type by
	 * caling `calc()`
	 */
	operator Result(void) const {
		return self().calc();
	}

	/**
	 * Apply the aggregator to a container
	 */
	template<typename Type>
	Class& apply(const Type& object) {
		apply_dispatch_(IsContainer<Type>(),object);
		return self();
	}

	/**
	 * Apply the aggregator to a container and calculate
	 */
	template<typename Type>
	Result run(const Type& object) {
		return apply(object).calc();
	}

private:

	template<typename Container>
	void apply_dispatch_(const std::true_type& is_container,Container container) {
		for(auto& value : container) self().append(value);
	}

	template<typename Queryable>
	void apply_dispatch_(const std::false_type& is_container,Queryable queryable) {
		queryable(self());
	}
};

#define STENCILA_AGGREGATOR_FUNCS(name,func)\
	name func(){ return name(); } \
	template<class Type> \
	name::result_type func(const Type& object){ return name().apply(object); }

/**
 * Dynamic aggregator class
 */
template<
	typename Value,
	typename Result
>
class AggregatorDynamic : public Clause {
public:
	virtual AggregatorDynamic* append(const Value& value) = 0;
	virtual Result result(void) = 0;
};


template<
	typename Function
>
class Each : public Aggregator<Each<Function>,void> {
private:
	Function func_;

public:

	virtual std::string code(void) const{
		return "each";
	}

	Each(Function func):
		func_(func){}

	template<class Type>
	Each& add(const Type& value){
		func_(value);
		return *this;
	}

	void calc(void) const {
	}
};

template<typename Function>
Each<Function> each(Function function){
	return Each<Function>(function);
}

template<class Type,typename Function>
void each(const Type& object, Function function){
	return Each<Function>(function).run(object);
}


class Count : public Aggregator<Count,unsigned int> {
protected:
	double count_;
	
public:
	Count(void):
		count_(0){
	}

	virtual std::string code(void) const{
		return "count";
	}

	template<class Type>
	Count& add(const Type& value){
		count_++;
		return *this;
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

	double calc(void) const {
		return count_;
	}
};

STENCILA_AGGREGATOR_FUNCS(Count,count)


class Sum : public Aggregator<Sum,double> {
protected:
	double sum_;
	
public:
	Sum(void):
		sum_(0){
	}

	virtual std::string code(void) const{
		return "sum";
	}

	template<class Type>
	Sum& add(const Type& value){
		sum_ += value;
		return *this;
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

	double calc(void) const {
		return sum_;
	}
};

STENCILA_AGGREGATOR_FUNCS(Sum,sum)

class SumDouble : public AggregatorDynamic<double,double> , public Sum {
	virtual std::string code(void) const{
		return "sum";
	}
	virtual SumDouble* append(const double& value) {
		Sum::append(value);
		return this;
	}
	virtual double result(void){
		return Sum::result();
	}
};


class Product : public Aggregator<Product,double> {
protected:
	double prod_;
	
public:
	Product(void):
		prod_(1){
	}

	virtual std::string code(void) const{
		return "prod";
	}

	template<class Type>
	Product& add(const Type& value){
		prod_ *= value;
		return *this;
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

	double calc(void) const {
		return prod_;
	}
};

STENCILA_AGGREGATOR_FUNCS(Product,prod)


class Mean : public Aggregator<Mean> {
private:
	double sum_;
	double count_;

public:
	Mean(void):
		sum_(0),count_(0){
	}

	virtual std::string code(void) const{
		return "mean";
	}

	template<class Type>
	Mean& add(const Type& value){
		sum_ += value;
		count_++;
		return *this;
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

	double calc(void) const {
		return sum_/count_;
	}
};

class GeometricMean : public Aggregator<GeometricMean> {
private:

	Mean mean_;

public:

	virtual std::string code(void) const{
		return "geomean";
	}

	template<class Type>
	GeometricMean add(const Type& value){
		if(value>0) mean_.append(value);
		return *this;
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

	double calc(void) const {
		return std::exp(mean_.calc());
	}
};

STENCILA_AGGREGATOR_FUNCS(GeometricMean,geomean)


#undef STENCILA_AGGREGATOR_FUNCS

// Forward declaration of By class.
// Given many template arguments to allow for Bys specialised
// for Array dimensions (see `array.hpp`)
template<
	class Arg1,
	class Arg2,
	class Arg3,
	class Arg4,
	class Arg5,
	class Arg6,
	class Arg7,
	class Arg8,
	class Arg9,
	class Arg10
>
class By;

}
