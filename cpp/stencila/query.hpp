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


template<
	class Class,
	typename Result = double
>
class Aggregate {
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

#define STENCILA_AGGREGATE_FUNCS(name,func)\
	name func(){ return name(); } \
	template<class Type> \
	name::result_type func(const Type& object){ return name().apply(object); }

/**
 * Dynamic aggregate class
 */
template<
	typename Value,
	typename Result
>
class Aggregater : public Clause {
public:
	virtual Aggregater* append(const Value& value) = 0;
	virtual Result result(void) = 0;
};


template<
	typename Function
>
class Each : public Aggregate<Each<Function>,void> {
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


class Count : public Aggregate<Count,unsigned int> {
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

STENCILA_AGGREGATE_FUNCS(Count,count)

class Counter :  public Clause, public Count {
public:
	virtual std::string code(void) const {
		return "count";
	}
	Counter* append(void) {
		Count::append(1);
		return this;
	}
	unsigned int result(void){
		return Count::result();
	}
};


class Sum : public Aggregate<Sum,double> {
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

STENCILA_AGGREGATE_FUNCS(Sum,sum)

class Summer : public Aggregater<double,double> , public Sum {
public:
	virtual std::string code(void) const{
		return "sum";
	}
	virtual Summer* append(const double& value) {
		Sum::append(value);
		return this;
	}
	virtual double result(void){
		return Sum::result();
	}
};


class Product : public Aggregate<Product,double> {
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

STENCILA_AGGREGATE_FUNCS(Product,prod)


class Mean : public Aggregate<Mean> {
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

class GeometricMean : public Aggregate<GeometricMean> {
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

STENCILA_AGGREGATE_FUNCS(GeometricMean,geomean)


#undef STENCILA_AGGREGATOR_FUNCS


class Where : public Clause {
public:


};


/**
 * `by` query specialised for Arrays.
 *
 * This `By` class contains dimensions which can
 * be used by Array to optimise it's running.
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