#pragma once

#include <fstream>

#include <stencila/exception.hpp>
#include <stencila/dimension.hpp>
#include <stencila/query.hpp>
#include <stencila/traits.hpp>

namespace Stencila {

/**
 * A dynamic array
 *
 * This implementation of array is useful for arrays of variable size.
 * It is a wrapper around the C++ std::vector class but has an interface that
 * is consistent as possible with static Arrays.
 */
template<
	typename Type = double
>
class Array {
private:

	std::vector<Type> values_;

public:
    
   	Array(void){
	}

    Array(const int& size):
    	values_(size){
    }

    Array(const int& size, const Type& value):
    	values_(size){
    	for(auto& item : *this) item = value;
    }

	/**
	 * Construct from an initializer_list (e.g. `{1.23,3.14,5.98}`)
	 */
    template<class Value>
	Array(const std::initializer_list<Value>& values){
        construct_(std::true_type(),values);
    }

    template<class Other>
    Array(const Other& other){
    	construct_(IsContainer<Other>(),other);
    }

private:

 	template<class Other>
    void construct_(const std::false_type& is_not_container,const Other& other){
        // Convert to size
        unsigned int num = other;
        size(num);
    }

 	template<class Other>
    void construct_(const std::true_type& is_container,const Other& other){
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

    operator std::vector<Type>() {
    	return values_;
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
    
 	void write(std::ostream& stream,const std::string format="tsv") const {

	}
    
};

} //namespace Stencila
