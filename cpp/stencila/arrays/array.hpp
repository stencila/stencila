#pragma once

#include <initializer_list>
#include <array>
#include <vector>
#include <iostream>

#include "dimension.hpp"
#include "query.hpp"

namespace Stencila {
namespace Arrays {

template<
	class Type
>
class Array<Type> {
private:

	std::vector<Type> values_;

public:
    
    Array(unsigned int size=0){
        resize(size);
    }
    
    Array(std::initializer_list<Type> values){
        unsigned int index = 0;
        for(auto iter=values.begin();iter!=values.end();iter++){
            if(index<size()) values_[index] = *iter;
            else values_.push_back(*iter);
            index++;
        }
    }
    
    unsigned int size(void) const {
        return values_.size();
    }
    
    void resize(unsigned int size) {
        return values_.resize(size);
    }
    
    void append(const Type& item) {
        return values_.push_back(item);
    }
    
    Type& operator()(int index){
        return values_[index];
    }
    
    const Type& operator()(int index) const {
        return values_[index];
    }
    
    Type& operator[](int index) {
        return values_[index];
    }

    const Type& operator[](int index) const {
        return values_[index];
    }
    
};

template<
	class Type,
	class Dim1,
	class Dim2,
	class Dim3,
	class Dim4,
	class Dim5,
	class Dim6
>
class Array {
private:
	static const uint size_ = 
		Dim1::size * 
		Dim2::size *
		Dim3::size *
		Dim4::size *
		Dim5::size *
		Dim6::size
	;

	Type values_[size_];

public:

	Array(void){
	}

	Array(const Type& other){
		for(Type& value : values_) value = other;
	}

	template<
		class Other
	>
    Array(const std::initializer_list<Other>& others){
        uint index = 0;
        for(auto& item : others){
            values_[index] = item;
            index++;
            if(index>=size()) break;
        }
    }

   	template<
		class Other,
		unsigned long Size
	>
    Array(const std::array<Other,Size>& array){
        uint index = 0;
        for(auto& item : array){
            values_[index] = item;
            index++;
            if(index>=size()) break;
        }
    }

   	template<
		class Other
	>
    Array(const std::vector<Other>& vector){
        uint index = 0;
        for(auto& item : vector){
            values_[index] = item;
            index++;
            if(index>=size()) break;
        }
    }

	void operator=(Type (*func)(uint)){
		for(uint index=0;index<size();index++) values_[index] = func(index);
	}

	void operator=(Type (*func)(uint,uint)){
		for(uint index=0;index<size();index++){
			values_[index] = func(level(Dim1(),index),level(Dim2(),index));
		}
	}

	uint size(void) const {
		return size_;
	}

	//! @{
	
	template<
		uint Order
	>
	class Rank {
	public:
		static const uint order(void){
			return Order;
		};
	};

	static const Rank<0> rank0;
	static const Rank<1> rank1;
	static const Rank<2> rank2;
	static const Rank<3> rank3;
	static const Rank<4> rank4;
	static const Rank<5> rank5;
	static const Rank<6> rank6;

	Rank<1> rank(Dim1) const { return rank1; }
	Rank<2> rank(Dim2) const { return rank2; }
	Rank<3> rank(Dim3) const { return rank3; }
	Rank<4> rank(Dim4) const { return rank4; }
	Rank<5> rank(Dim5) const { return rank5; }
	Rank<6> rank(Dim6) const { return rank6; }
	
	template<class Dim>
	Rank<0> rank(Dim) const { return rank0; }

	//! @}
	
	//! @{

	uint base(Rank<1>) const { 
		return Dim2::size * Dim3::size * Dim4::size * Dim5::size * Dim6::size;
	}
	uint base(Rank<2>) const { 
		return Dim3::size * Dim4::size * Dim5::size * Dim6::size;
	}
	uint base(Rank<3>) const { 
		return Dim4::size * Dim5::size * Dim6::size;
	}
	uint base(Rank<4>) const { 
		return Dim5::size * Dim6::size;
	}
	uint base(Rank<5>) const { 
		return Dim6::size;
	}
	uint base(Rank<6>) const { 
		return 1;
	}
	template<uint Order>
	uint base(Rank<Order>) const { 
		return 0;
	}

	//! @}

	uint index(
		uint level1,
		uint level2 = 0,
		uint level3 = 0,
		uint level4 = 0,
		uint level5 = 0,
		uint level6 = 0
	) const {
		return 
			level1 * base(rank1) +
			level2 * base(rank2) +
			level3 * base(rank3) +
			level4 * base(rank4) +
			level5 * base(rank5) +
			level6
		;
	}

	uint level(Dim1,uint index) const {
		return index/base(rank1);
	}
	uint level(Dim2,uint index) const {
		return index/base(rank2)%Dim2::size;
	}
	uint level(Dim3,uint index) const {
		return index/base(rank3)%Dim3::size;
	}
	uint level(Dim4,uint index) const {
		return index/base(rank4)%Dim4::size;
	}
	uint level(Dim5,uint index) const {
		return index/base(rank5)%Dim5::size;
	}
	uint level(Dim6,uint index) const {
		return index/base(rank6)%Dim6::size;
	}

	template<class Dim>
	uint level(Dim,uint index) const {
		return 0;
	}

	uint level(uint dim,uint index) const {
		switch(dim){
			case 0: return level(Dim1(),index); break;
			case 1: return level(Dim2(),index); break;
			case 2: return level(Dim3(),index); break;
			case 3: return level(Dim4(),index); break;
			case 4: return level(Dim5(),index); break;
			case 5: return level(Dim6(),index); break;
			default:
				return 0;
			break;
		}
	}

	//!@}
	
	template<
		typename Value
	>
	class Iterator {
	public:

		const Iterator& operator++() {
			++index_;
			return *this;
		}

		Iterator operator++(int){
			Iterator copy(*this);
			++index_;
			return copy;
		}

		bool operator==(const Iterator& other) const {
			return index_ == other.index_;
		}

		bool operator!=(const Iterator& other) const {
			return index_ != other.index_;
		}

		Value& operator*() const{ 
			return value();
		}

		uint index(void) const {
			return index_;
		}

		uint level(uint dim) const {
			return array_->level(dim,index_);
		}

		Value& value(void) const{ 
			return values_[index_];
		}

	protected:
		Iterator(uint start,Value* values,const Array* array):
			index_(start),
			values_(values),
			array_(array){
		}

		friend class Array;

	private:
		uint index_;
		Value* values_;
		const Array* array_;
	};

	Iterator<Type> begin(void) {
		return Iterator<Type>(0,values_,this);
	}

	Iterator<const Type> begin(void) const {
		return Iterator<const Type>(0,values_,this);
	}

	Iterator<Type> end(void) {
		return Iterator<Type>(size_,values_,this);
	}

	Iterator<const Type> end(void) const {
		return Iterator<const Type>(size_,values_,this);
	}

	//!@{
	
	Type& operator[](uint index){
		return values_[index];
	}

	const Type& operator[](uint index) const {
		return values_[index];
	}

	Type& operator()(
		uint level1 = 0,
		uint level2 = 0,
		uint level3 = 0,
		uint level4 = 0,
		uint level5 = 0,
		uint level6 = 0
	){
		return values_[index(level1,level2,level3,level4,level5,level6)];
	}

	const Type& operator()(
		uint level1 = 0,
		uint level2 = 0,
		uint level3 = 0,
		uint level4 = 0,
		uint level5 = 0,
		uint level6 = 0
	) const {
		return values_[index(level1,level2,level3,level4,level5,level6)];
	}

	//!@}

	double operator()(Count count) const {
		return count.aggregate(*this);
	}

	double operator()(Sum sum) const {
		return sum.aggregate(*this);
	}

	//double operator()(Mean mean){
	//	return mean.aggregate(*this);
	//}

	//double operator()(Geomean geomean){
	//	return geomean.aggregate(*this);
	//}

	template<
		class DimA,
		class DimB,
		class DimC,
		class DimD,
		class DimE,
		class DimF,
		class Aggregator
	>
	Array<double,DimA,DimB,DimC,DimD,DimE,DimF> operator()(const By<DimA,DimB,DimC,DimD,DimE,DimF>& by,const Aggregator& aggregator){
		Array<Aggregator,DimA,DimB,DimC,DimD,DimE,DimF> aggregators;

		for(int index=0;index<size();index++) {
			aggregators(
				level(DimA(),index),
				level(DimB(),index),
				level(DimC(),index),
				level(DimD(),index),
				level(DimE(),index),
				level(DimF(),index)
			).append(
				values_[index]
			);
		}

		Array<double,DimA,DimB,DimC,DimD,DimE,DimF> result;
		for(int index=0;index<aggregators.size();index++) result[index] = aggregators[index].finalise();
		return result;
	}

	template<
		class DimA,class DimB,class DimC,class DimD,class DimE,class DimF
	>
	Array<double,DimA,DimB,DimC,DimD,DimE,DimF> operator()(
		const By<DimA,DimB,DimC,DimD,DimE,DimF>& by
	){
		return operator()(by,sum());
	}

	template<
		class Other,
		class DimA,class DimB,class DimC,class DimD,class DimE,class DimF
	>
	Array<double,Dim1,Dim2,Dim3,Dim4,Dim5,Dim6> operator*(
		const Array<Other,DimA,DimB,DimC,DimD,DimE,DimF>& other
	) const {
		Array<double,Dim1,Dim2,Dim3,Dim4,Dim5,Dim6> result;
		for(int index=0;index<size();index++) {
			result[index] = (*this)[index] * other.correlate(
				index,
				Dim1(),Dim2(),Dim3(),Dim4(),Dim5(),Dim6()
			);
		}
		return result;
	}

	void write(std::ostream& stream) const {
		// Write header row
		if(Dim1::size>1) stream<<Dim1::label<<"\t";
		if(Dim2::size>1) stream<<Dim2::label<<"\t";
		if(Dim3::size>1) stream<<Dim3::label<<"\t";
		if(Dim4::size>1) stream<<Dim4::label<<"\t";
		if(Dim5::size>1) stream<<Dim5::label<<"\t";
		if(Dim6::size>1) stream<<Dim6::label<<"\t";
		stream<<"value"<<std::endl;

		for(uint index = 0; index<size(); index++){
			if(Dim1::size>1) stream<<level(Dim1(),index)<<"\t";
			if(Dim2::size>1) stream<<level(Dim2(),index)<<"\t";
			if(Dim3::size>1) stream<<level(Dim3(),index)<<"\t";
			if(Dim4::size>1) stream<<level(Dim4(),index)<<"\t";
			if(Dim5::size>1) stream<<level(Dim5(),index)<<"\t";
			if(Dim6::size>1) stream<<level(Dim6(),index)<<"\t";
			stream<<values_[index]<<std::endl;
		}
	}

	void write(const std::string& filename) const {
		std::ofstream file(filename);
		write(file);
	}

};

template<
	class Type,
	class... Dims
>
std::ostream& operator<<(std::ostream& stream, const Array<Type,Dims...>& array){
	array.write(stream);
	return stream;
}

}
}
