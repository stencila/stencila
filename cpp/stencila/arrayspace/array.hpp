#pragma once

#include <initializer_list>
#include <array>
#include <vector>
#include <iostream>

#include "query.hpp"

namespace Stencila {
namespace Arrayspace {


template<
	unsigned int Order
>
class Rank {
public:
	static const unsigned int order;
};

template<
	unsigned int Order
>
const unsigned int Rank<Order>::order = Order;

template<
	class Type,
	class Dimension1,
	class Dimension2,
	class Dimension3,
	class Dimension4,
	class Dimension5,
	class Dimension6
>
class Array {
private:
	static const unsigned int size_ = 
		Dimension1::size * 
		Dimension2::size *
		Dimension3::size *
		Dimension4::size *
		Dimension5::size *
		Dimension6::size
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
        unsigned int index = 0;
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
        unsigned int index = 0;
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
        unsigned int index = 0;
        for(auto& item : vector){
            values_[index] = item;
            index++;
            if(index>=size()) break;
        }
    }

	unsigned int size(void) const {
		return size_;
	}

	unsigned int index(unsigned int level1,unsigned int level2=0,unsigned int level3=0,unsigned int level4=0,unsigned int level5=0,unsigned int level6=0) const {
		return 
			level1 * (Dimension2::size * Dimension3::size * Dimension4::size * Dimension5::size * Dimension6::size) +
			level2 * (                   Dimension3::size * Dimension4::size * Dimension5::size * Dimension6::size) +
			level3 * (                                      Dimension4::size * Dimension5::size * Dimension6::size) +
			level4 * (                                                         Dimension5::size * Dimension6::size) +
			level5 * (                                                                            Dimension6::size) +
			level6
		;
	}

	/*
	translate(index,rank(dimA),dimA,dimB,dimC,dimD,dimE,dimF) +
	translate(index,rank(dimB),dimA,dimB,dimC,dimD,dimE,dimF) +
	translate(index,rank(dimC),dimA,dimB,dimC,dimD,dimE,dimF) +
	translate(index,rank(dimD),dimA,dimB,dimC,dimD,dimE,dimF) +
	translate(index,rank(dimE),dimA,dimB,dimC,dimD,dimE,dimF) +
	translate(index,rank(dimF),dimA,dimB,dimC,dimD,dimE,dimF);
	*/

	unsigned int level(Dimension1,unsigned int index) const {
		return index/(
			Dimension2::size *
			Dimension3::size *
			Dimension4::size *
			Dimension5::size *
			Dimension6::size
		);
	}

	unsigned int level(Dimension2,unsigned int index) const {
		return index/(
			Dimension3::size *
			Dimension4::size *
			Dimension5::size *
			Dimension6::size
		)%Dimension2::size;
	}

	unsigned int level(Dimension3,unsigned int index) const {
		return index/(
			Dimension4::size *
			Dimension5::size *
			Dimension6::size
		)%Dimension3::size;
	}

	unsigned int level(Dimension4,unsigned int index) const {
		return index/(
			Dimension5::size *
			Dimension6::size
		)%Dimension4::size;
	}

	unsigned int level(Dimension5,unsigned int index) const {
		return index/(
			Dimension6::size
		)%Dimension5::size;
	}

	unsigned int level(Dimension6,unsigned int index) const {
		return index%Dimension6::size;
	}

	template<
		class Dim
	>
	unsigned int level(Dim,unsigned int index) const {
		return 0;
	}

	//!@}
	
	Rank<1> rank(Dimension1) const { return Rank<1>(); }
	Rank<2> rank(Dimension2) const { return Rank<2>(); }
	Rank<3> rank(Dimension3) const { return Rank<3>(); }
	Rank<4> rank(Dimension4) const { return Rank<4>(); }
	Rank<5> rank(Dimension5) const { return Rank<5>(); }
	Rank<6> rank(Dimension6) const { return Rank<6>(); }
	
	template<
		class Dim
	>
	Rank<0> rank(Dim) const { return Rank<0>(); }


	Type* begin(void) {
		return values_;
	}

	const Type* begin(void) const {
		return values_;
	}

	Type* end(void) {
		return values_+size_;
	}

	const Type* end(void) const {
		return values_+size_;
	}

	//!@{
	
	Type& operator[](unsigned int index){
		return values_[index];
	}

	const Type& operator[](unsigned int index) const {
		return values_[index];
	}

	//!@}

	void set(Type (*func)(void)){
		for(Type& value : values_) value = func();
	}

	void set(Type (*func)(unsigned int)){
		for(unsigned int index=0;index<size();index++) values_[index] = func(index);
	}

	Type& operator()(
		unsigned int level1 = 0,
		unsigned int level2 = 0,
		unsigned int level3 = 0,
		unsigned int level4 = 0,
		unsigned int level5 = 0,
		unsigned int level6 = 0
	){
		return values_[index(level1,level2,level3,level4,level5,level6)];
	}

	const Type& operator()(
		unsigned int level1 = 0,
		unsigned int level2 = 0,
		unsigned int level3 = 0,
		unsigned int level4 = 0,
		unsigned int level5 = 0,
		unsigned int level6 = 0
	) const {
		return values_[index(level1,level2,level3,level4,level5,level6)];
	}

	void operator()(Type (*func)(void)){
		set(func);
	}

	void operator()(Type (*func)(unsigned int)){
		set(func);
	}

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
	Array<double,DimA,DimB,DimC,DimD,DimE,DimF> operator()(const By<DimA,DimB,DimC,DimD,DimE,DimF>& by){
		return operator()(by,sum());
	}

	template<
		class Other,
		class DimA,class DimB,class DimC,class DimD,class DimE,class DimF
	>
	Array<double,Dimension1,Dimension2,Dimension3,Dimension4,Dimension5,Dimension6> operator*(const Array<Other,DimA,DimB,DimC,DimD,DimE,DimF>& other) const {
		Array<double,Dimension1,Dimension2,Dimension3,Dimension4,Dimension5,Dimension6> result;
		for(int index=0;index<size();index++) {
			result[index] = (*this)[index] * other.correlate(
				index,
				Dimension1(),Dimension2(),Dimension3(),Dimension4(),Dimension5(),Dimension6()
			);
		}
		return result;
	}

	void write(std::ostream& stream) const {
		// Write header row
		if(Dimension1::size>1) stream<<Dimension1::label<<"\t";
		if(Dimension2::size>1) stream<<Dimension2::label<<"\t";
		if(Dimension3::size>1) stream<<Dimension3::label<<"\t";
		if(Dimension4::size>1) stream<<Dimension4::label<<"\t";
		if(Dimension5::size>1) stream<<Dimension5::label<<"\t";
		if(Dimension6::size>1) stream<<Dimension6::label<<"\t";
		stream<<"value"<<std::endl;

		for(unsigned int index = 0; index<size(); index++){
			if(Dimension1::size>1) stream<<level(Dimension1(),index)<<"\t";
			if(Dimension2::size>1) stream<<level(Dimension2(),index)<<"\t";
			if(Dimension3::size>1) stream<<level(Dimension3(),index)<<"\t";
			if(Dimension4::size>1) stream<<level(Dimension4(),index)<<"\t";
			if(Dimension5::size>1) stream<<level(Dimension5(),index)<<"\t";
			if(Dimension6::size>1) stream<<level(Dimension6(),index)<<"\t";
			stream<<values_[index]<<std::endl;
		}
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
