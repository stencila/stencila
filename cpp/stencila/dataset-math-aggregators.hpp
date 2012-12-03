/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file dataset-math-aggregators.hpp
//! @brief Definition of aggregate functions for a Dataset

#pragma once

#include <string>
#include <map>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

#include <stencila/sqlite.hpp>

namespace Stencila {
namespace MathAggregators {

//! @{
//! @brief Location descriptive statistics

class Mean {
protected:
    unsigned long int count_;
    double sum_;

public:
	Mean(void):
		count_(0),
		sum_(0){
	}
		
        
	unsigned long int count(void) const {
		return count_;
	}
    
	double sum(void) const {
		return sum_;
	}
	
	void append(const double& value){
		count_++;
		sum_ += value;
	}
	
	std::string dump(void){
		char value[1000];
		std::sprintf(value, "%li,%lf", count(), sum());
		return value;
	}
	
	void load(const std::string& value){
		std::sscanf(value.c_str(), "%li,%lf", &count_, &sum_);
	}
	
	void combine(const Mean& other){
		count_ += other.count();
		sum_ += other.sum();
	}	
	
	double calc(void) const {
		return sum()/count();
	}
};

class GeometricMean : public Mean {
public:
    void append(const double& value){
        Mean::append(std::log(value));
    }
    
    double calc(void) const {
        return std::exp(Mean::calc());
    }
};

class HarmonicMean : public Mean {
public:
    void append(const double& value){
        Mean::append(1/value);
    }
    
    double calc(void) const {
        return count()/sum();
    }
};

//! @}

//! @{
//! @brief Dispersion descriptive statistics

class Variance {
/*
    n = 0
    mean = 0
    M2 = 0
 
    for x in data:
        n = n + 1
        delta = x - mean
        mean = mean + delta/n
        M2 = M2 + delta*(x - mean)
 
    variance = M2/(n - 1)
*/
protected:
	unsigned long int count_;
    double mean_;
	double m2_;

public:
	Variance(void):
		count_(0),
		mean_(0),
		m2_(0){
	}
		
        
	unsigned long int count(void) const {
		return count_;
	}
    
	double mean(void) const {
		return mean_;
	}
	
	double m2(void) const {
		return m2_;
	}
	
	void append(const double& value){
		count_++;
		double delta = value - mean_;
		mean_ += delta/count_;
		m2_ += delta*(value-mean_);
	}
	
	std::string dump(void){
		char value[1000];
		std::sprintf(value, "%li,%lf,%lf", count(), mean(), m2());
		return value;
	}
	
	void load(const std::string& value){
		std::sscanf(value.c_str(), "%li,%lf,%lf", &count_, &mean_, &m2_);
	}
	
	void combine(const Variance& other){
		count_ += other.count();
		mean_ += other.mean();
		m2_ += other.m2();
	}	
	
	double calc(void) const {
		return m2()/(count() - 1);
	}
};

class StandardDeviation : public Variance {
public:
	double calc(void) const {
		return std::sqrt(Variance::calc());
	}
};

//! @}

template<typename Aggregator>
static void append(sqlite3_context* context, int argc, sqlite3_value** argv){
	sqlite3_value* value = argv[0];
	if(sqlite3_value_numeric_type(value)!=SQLITE_NULL){
		Aggregator* agg = static_cast<Aggregator*>(sqlite3_aggregate_context(context, sizeof(Aggregator)));
		agg->append(sqlite3_value_double(value));
	}
}

template<typename Aggregator>
static void store(sqlite3_context* context){
	Aggregator* agg = static_cast<Aggregator*>(sqlite3_aggregate_context(context, sizeof(Aggregator)));
	std::string value = agg->dump();
	sqlite3_result_text(context,value.c_str(),value.length(),0);
}

template<typename Aggregator>
static void combine(sqlite3_context* context, int argc, sqlite3_value** argv){
	//Convert first argument to a std::string. Using a stringstream
	//is the only approach to this that worked for me.
	std::stringstream stream;
	stream<<sqlite3_value_text(argv[0]);
	std::string dump = stream.str();
	//Load dump into accumulator
	Aggregator stored;
	stored.load(dump);
	//Add to current accumulator
	Aggregator* agg = static_cast<Aggregator*>(sqlite3_aggregate_context(context, sizeof(Aggregator)));
	agg->combine(stored);
}

template<typename Aggregator>
static void calc(sqlite3_context* context){
	Aggregator* agg = static_cast<Aggregator*>(sqlite3_aggregate_context(context, sizeof(Aggregator)));
	sqlite3_result_double(context,agg->calc());
}

#define STENCILA_LOCAL(NAME,AGGREGATOR) \
	sqlite3_create_function(db, #NAME, 1, SQLITE_UTF8, 0, 0, append<AGGREGATOR>, calc<AGGREGATOR>); \
	sqlite3_create_function(db, #NAME"__1", 1, SQLITE_UTF8, 0, 0, append<AGGREGATOR>, store<AGGREGATOR>); \
	sqlite3_create_function(db, #NAME"__2", 1, SQLITE_UTF8, 0, 0, combine<AGGREGATOR>, calc<AGGREGATOR>); \

inline void create(sqlite3* db) {
    //This list includes commented lines for builtin SQLite functions at http://www.sqlite.org/lang_aggfunc.html
    //That is so this list can be used to constuct Dataquery call elements in R, Python etc packages

    //count
    //min
    //max
    //sum
    
    //avg
    STENCILA_LOCAL(mean,Mean)
    STENCILA_LOCAL(geomean,GeometricMean)
    STENCILA_LOCAL(harmean,HarmonicMean)

    STENCILA_LOCAL(var,Variance)
    STENCILA_LOCAL(sd,StandardDeviation)
}

#undef STENCILA_LOCAL

}
}
