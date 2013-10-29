/*
Copyright (c) 2013 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/arrayspace/arrayspace.hpp>

BOOST_AUTO_TEST_SUITE(arrayspace)

using namespace Stencila::Arrayspace;

STENCILA_ARRAY_DIM(Area,areas,area,3)
STENCILA_ARRAY_DIM(Age,ages,age,3)
STENCILA_ARRAY_DIM(Sex,sexes,sex,2)
STENCILA_ARRAY_DIM(Dummy,dummys,dummy,2);

Array<double,Area,Age> numbers;

BOOST_AUTO_TEST_CASE(dimensions){

	BOOST_CHECK_EQUAL(Area::size,3);
	BOOST_CHECK_EQUAL(Area::label,"area");

}

BOOST_AUTO_TEST_CASE(array_constructors){

	{
		Array<> array;

		BOOST_CHECK_EQUAL(array.size(),1);
	}

	{
		Array<> array = 3.14;

		BOOST_CHECK_EQUAL(array.size(),1);
		BOOST_CHECK_EQUAL(array[0],3.14);
	}

	{
		Array<double,Age> array = {1,2,3};

		BOOST_CHECK_EQUAL(array.size(),3);
		BOOST_CHECK_EQUAL(array[0],1);
		BOOST_CHECK_EQUAL(array[1],2);
		BOOST_CHECK_EQUAL(array[2],3);
	}

	{
		std::array<double,3> std_array = {1,2,3};
		Array<double,Age> array = std_array;

		BOOST_CHECK_EQUAL(array.size(),3);
		BOOST_CHECK_EQUAL(array[0],1);
		BOOST_CHECK_EQUAL(array[1],2);
		BOOST_CHECK_EQUAL(array[2],3);
	}

}

BOOST_AUTO_TEST_CASE(array_assignment){

	Array<double,Age> array;

	array = 1;
	array = 1.0;
	array = {1,2,3};

	std::array<double,3> arr;
	array = arr;

	std::vector<double> vec(3);
	array = vec;

}

BOOST_AUTO_TEST_CASE(array_ranks){

	Array<double,Age,Area> array;

	BOOST_CHECK_EQUAL(array.rank(ages).order,1);
	BOOST_CHECK_EQUAL(array.rank(areas).order,2);
	BOOST_CHECK_EQUAL(array.rank(sexes).order,0);

	Rank<1> rank = array.rank(ages);
	
}

BOOST_AUTO_TEST_CASE(queries){

	Array<double,Area,Age> numbers = 2;
	
	BOOST_CHECK_EQUAL(
		numbers(count()),
		numbers.size()
	);

	BOOST_CHECK_EQUAL(
		numbers(sum()),
		2*numbers(count())
	);

	#if 0
	{
		Array<double,Area> counts = numbers(by(areas),count());
		BOOST_CHECK_EQUAL(counts(0),3);
		BOOST_CHECK_EQUAL(counts(1),3);
		BOOST_CHECK_EQUAL(counts(2),3);
	}

	{
		Array<double,Area> sums = numbers(by(areas),sum());
		BOOST_CHECK_EQUAL(sums(0),6);
		BOOST_CHECK_EQUAL(sums(1),6);
		BOOST_CHECK_EQUAL(sums(2),6);
	}

	{
		Array<double,Area> sums = numbers(by(areas));
		BOOST_CHECK_EQUAL(sums(0),6);
		BOOST_CHECK_EQUAL(sums(1),6);
		BOOST_CHECK_EQUAL(sums(2),6);
	}

	{
		Array<double,Area,Age> sums = numbers(by(areas,ages),sum());
		BOOST_CHECK_EQUAL(sums(0,0),2);
		BOOST_CHECK_EQUAL(sums(0,1),2);
		BOOST_CHECK_EQUAL(sums(0,2),2);
	}

	{
		//Array<double,Area,Age> sums = numbers(by(areas),count(),where(ages==1));
		//BOOST_CHECK_EQUAL(sums(0,0),2);
		//BOOST_CHECK_EQUAL(sums(0,1),2);
		//BOOST_CHECK_EQUAL(sums(0,2),2);
	}
	#endif
}

BOOST_AUTO_TEST_SUITE_END()

/*

	numbers = {
		0,1,0,0,0,0,
		0,0,2,3,0,0,
		0,0,0,0,4,0
	};

	//numbers(0,1) = 3;

	std::cout<<numbers<<std::endl;
	
	for(int area : areas){
		for(int age : ages){
			std::cout<<area<<"\t"<<age<<"\t"<<"\t"<<numbers.index(area,age)<<"\t"<<numbers(area,age)<<std::endl;
		}
	}
	
	std::cout<<numbers(sum())<<std::endl;


	double sum = 0;
	numbers(func([&](double value){
        sum += value;
    }));
    std::cout<<sum<<std::endl;

    std::cout<<numbers(by(areas))<<std::endl;
    std::cout<<numbers(by(ages,sexes))<<std::endl;
    std::cout<<numbers(by(dummys))<<std::endl;

}

*/