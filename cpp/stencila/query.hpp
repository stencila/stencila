#pragma once

#include <stencila/traits.hpp>
#include <stencila/dimension.hpp>

namespace Stencila {

/**
 * An element of a Query
 */
class Clause {
public:

	/**
	 * Get the code representation of the clause
	 */
    virtual std::string code(void) const = 0;

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
class Aggregate : public AggregateDynamic<Values,Result> {
public:

	typedef Result result_type;
	
	/**
	 * Convienience method to return address as derived class
	 */
	const Derived& self(void) const {
		return static_cast<const Derived&>(*this);
	}

	/**
	 * Convienience method to return address as derived class
	 */
	Derived& self(void) {
		return static_cast<Derived&>(*this);
	}

	/**
	 * Append an item
	 */
	template<class Type>
	Derived& append(const Type& value){
		self().append_static(value);
		return self();
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
    Derived& join(const Derived& other){
        return self();
    }

    /**
     * Get the result of the aggregator
     */
	Result result(void) const {
		return self().result_static();
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

	/**
	 * Apply the aggregator to a container
	 */
	template<typename Type>
	Derived& apply(const Type& object) {
		apply_dispatch_(IsContainer<Type>(),object);
		return self();
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

#define STENCILA_AGGREGATE_FUNCS(name,func)\
	static name func(){ return name(); } \
	template<class Type> static name::result_type func(const Type& object){ return name().apply(object).result(); } \


template<
	typename Function
>
class Each : public Aggregate<Each<Function>,bool,void> {
private:

	Function function_;

public:

	virtual std::string code(void) const{
		return "each";
	}

	Each(Function function):
		function_(function){}

	template<class Type>
	Each& append_static(const Type& value){
		function_(value);
		return *this;
	}

	void result_static(void) const {
	}

};

template<typename Function>
Each<Function> each(Function function){
	return Each<Function>(function);
}

template<class Type,typename Function>
void each(const Type& object, Function function){
	return Each<Function>(function).apply(object).result();
}


class Count : public Aggregate<Count,double,uint> {
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

STENCILA_AGGREGATE_FUNCS(Count,count)

class Sum : public Aggregate<Sum,double,double> {
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

STENCILA_AGGREGATE_FUNCS(Sum,sum)

class Product : public Aggregate<Product,double,double> {
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

STENCILA_AGGREGATE_FUNCS(Product,prod)


class Mean : public Aggregate<Mean,double,double> {
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

class GeometricMean : public Aggregate<GeometricMean,double,double> {
private:

	Mean mean_;

public:

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

class HarmonicMean : public Mean {
private:

	Mean mean_;

public:

	virtual std::string code(void) const{
		return "harmean";
	}

	template<class Type>
	void append_static(const Type& value){
		if(value!=0) Mean::append_static(1.0/value);
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
		return 1.0/Mean::result_static();
	}
};

STENCILA_AGGREGATE_FUNCS(GeometricMean,geomean)


#undef STENCILA_AGGREGATOR_FUNCS


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
