#pragma once

namespace Stencila {

class Query {
public:

};

template<
	class Class,
	typename Result = double
>
class Aggregator : public Query {
public:
	
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
		return self();
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
     * @param  value String representation
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
	Result calc(void) const {
		return Result();
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
	template<class Container>
	Class& apply(const Container& container) {
		for(auto& value : container) self().append(value);
		return self();
	}

	/**
	 * Apply and calculate the aggregator on a container
	 */
	template<class Container>
	Result run(const Container& container) {
		return self().apply(container).calc();
	}

};


template<
	typename Function
>
class Each : public Aggregator<Each<Function>,void> {
private:
	Function func_;

public:

	Each(Function func):
		func_(func){}

	template<class Type>
	Each& append(const Type& value){
		func_(value);
		return *this;
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

	template<class Type>
	Count& append(const Type& value){
		count_++;
		return *this;
	}

    std::string dump(void){
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

Count count(){
	return Count();
}

template<class Type>
double count(const Type& object){
	return Count().run(object);
}


class Sum : public Aggregator<Sum,double> {
protected:
	double sum_;
	
public:
	Sum(void):
		sum_(0){
	}

	template<class Type>
	Sum& append(const Type& value){
		sum_ += value;
		return *this;
	}

    std::string dump(void){
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

Sum sum(){
	return Sum();
}

template<class Type>
double sum(const Type& object){
	return Sum().run(object);
}

}
