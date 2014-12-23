#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/array-static.hpp>
#include <stencila/query.hpp>
#include <stencila/structure.hpp>

BOOST_AUTO_TEST_SUITE(array_static)

using namespace Stencila;

STENCILA_DIM(One,one,one,1);
STENCILA_DIM(Two,two,two,2);
STENCILA_DIM(Three,three,three,3);
STENCILA_DIM(Four,four,four,4);
STENCILA_DIM(Five,five,five,5);
STENCILA_DIM(Six,Sixe,six,6);
STENCILA_DIM(Seven,seven,seven,7);

BOOST_AUTO_TEST_CASE(constructors){
	typedef Array<double,Three> A;

	A a;

	A b(3.14);
	BOOST_CHECK_EQUAL(b[0],3.14);
	BOOST_CHECK_EQUAL(b[1],3.14);
	BOOST_CHECK_EQUAL(b[2],3.14);

	A c({6,7,9});
	BOOST_CHECK_EQUAL(c[0],6);
	BOOST_CHECK_EQUAL(c[1],7);
	BOOST_CHECK_EQUAL(c[2],9);

	std::vector<double> std_vector({1,2,3});
	A d(std_vector);
	BOOST_CHECK_EQUAL(d[0],std_vector[0]);
	BOOST_CHECK_EQUAL(d[1],std_vector[1]);
	BOOST_CHECK_EQUAL(d[2],std_vector[2]);

	std::array<double,3> std_array = {1,2,3};
	A e(std_array);
	BOOST_CHECK_EQUAL(e[0],std_array[0]);
	BOOST_CHECK_EQUAL(e[1],std_array[1]);
	BOOST_CHECK_EQUAL(e[2],std_array[2]);

	int jumper = 6;
	A f([&jumper](){return jumper++;});
	BOOST_CHECK_EQUAL(f[0],6);
	BOOST_CHECK_EQUAL(f[1],7);
	BOOST_CHECK_EQUAL(f[2],8);

	A g([](Level<Three> level){return level.index();});
	BOOST_CHECK_EQUAL(g[0],0);
	BOOST_CHECK_EQUAL(g[1],1);
	BOOST_CHECK_EQUAL(g[2],2);
}

BOOST_AUTO_TEST_CASE(size){
	Array<double,Three> a;
	BOOST_CHECK_EQUAL(a.size(),three.size());

	Array<double,Four,Five,Seven> b;
	BOOST_CHECK_EQUAL(b.size(),four.size()*five.size()*seven.size());
}

BOOST_AUTO_TEST_CASE(dimensioned){
	Array<double,Four,Five,Seven> a;

	BOOST_CHECK(a.dimensioned(four));
	BOOST_CHECK(a.dimensioned(seven));
	BOOST_CHECK(not a.dimensioned(two));
}

BOOST_AUTO_TEST_CASE(subscript){
	Array<double,One> a = {1};
	BOOST_CHECK_EQUAL(a(0),1);

	Array<double,One,Two> b = {11,12};
	BOOST_CHECK_EQUAL(b(0,0),11);
	BOOST_CHECK_EQUAL(b(0,1),12);
	
	Array<double,Two,Three> c = {11,12,13,21,22,23};
	BOOST_CHECK_EQUAL(c(0,1),12);
	BOOST_CHECK_EQUAL(c(1,0),21);
	BOOST_CHECK_EQUAL(c(1,1),22);
	BOOST_CHECK_EQUAL(c(1,2),23);

	// The following should not compile because they involve the
	// wrong number of levels, or levels in the wrong order:
	//   a(0,0);
	//   b(0);
	//   c(0,0);
	//(that's a feature, not a bug!)
}

// This test is causing a segfault, preventing other
// tests from running. So temporarily commented out.
#if 0
BOOST_AUTO_TEST_CASE(query){
	Array<int,Two,Five,Seven> a = 3;

	// Static queries
	BOOST_CHECK_EQUAL(count(a),a.size());
	Count counter;
	BOOST_CHECK_EQUAL(a(counter),a.size());
	BOOST_CHECK_EQUAL(sum(a),a.size()*3);

	// Dynamic queries
	BOOST_CHECK_EQUAL(a(new Count)[0],count(a));
	BOOST_CHECK_EQUAL(a(new Sum)[0],sum(a));    

	// Each aggregator
	Array<char,Four> b = {'f','o','r','d'};
	std::string word;
	each(b,[&word](char item){
		word += item;
	});
	BOOST_CHECK_EQUAL(word,"ford");
}
#endif

BOOST_AUTO_TEST_CASE(query_by){
	Array<double,Two,Three> numbers = 2;
	
	{
		Array<unsigned int,Two> counts = numbers(count,by(two));
		BOOST_CHECK_EQUAL(counts(0),3u);
		BOOST_CHECK_EQUAL(counts(1),3u);
	}
	{
		auto sums = numbers(sum,by(two));
		BOOST_CHECK_EQUAL(sums(0),6);
		BOOST_CHECK_EQUAL(sums(1),6);
	}
	{
		auto sums = numbers(sum,by(three));
		BOOST_CHECK_EQUAL(sums(0),4);
		BOOST_CHECK_EQUAL(sums(1),4);
		BOOST_CHECK_EQUAL(sums(2),4);
	}
	{
		auto sums = numbers(sum,by(two,three));
		BOOST_CHECK_EQUAL(sums(0,0),2);
		BOOST_CHECK_EQUAL(sums(0,1),2);
		BOOST_CHECK_EQUAL(sums(1,2),2);
	}
}

BOOST_AUTO_TEST_CASE(numeric_operators){
	Array<double,Three> numbers = {1,2,3};

	numbers /= 2;
	BOOST_CHECK_EQUAL(numbers(0),0.5);
	BOOST_CHECK_EQUAL(numbers(1),1);
	BOOST_CHECK_EQUAL(numbers(2),1.5);

	numbers += 1.5;
	BOOST_CHECK_EQUAL(numbers(0),2);
	BOOST_CHECK_EQUAL(numbers(1),2.5);
	BOOST_CHECK_EQUAL(numbers(2),3.0);

}

BOOST_AUTO_TEST_CASE(read){
	std::stringstream stream;
	stream.str("two\tvalue\n0\t2\n");
	stream.seekg(0);

	Array<int,Two> a = 3;
	a.read(stream);

	BOOST_CHECK_EQUAL(a[0],2);
	BOOST_CHECK_EQUAL(a[1],3);
}

BOOST_AUTO_TEST_CASE(write){
	// Create a grid....
	Array<int,Two,Three> a = 1;
	a[5] = 42;
	// Write to a stream
	std::ostringstream stream;
	a.write(stream);
	// Check the stream's contents
	std::string output = stream.str();
	std::vector<std::string> lines;
	boost::split(lines,output,boost::is_any_of("\n"));
	BOOST_CHECK_EQUAL(lines.size(),8u);
	BOOST_CHECK_EQUAL(lines[0],"two\tthree\tvalue");
	BOOST_CHECK_EQUAL(lines[1],"0\t0\t1");
	BOOST_CHECK_EQUAL(lines[5],"1\t1\t1");
	BOOST_CHECK_EQUAL(lines[6],"1\t2\t42");
}

struct A : public Structure<A> {

	int a = 1;
	int b = 2;

	template<class Mirror>
	void reflect(Mirror& mirror){
		mirror
			.data(a,"a")
			.data(b,"b")
		;
	}
};

BOOST_AUTO_TEST_CASE(write_reflect){
	Array<A,Three> a;
	a(1).a = 7373;
	// Write to stream
	std::ostringstream stream;
	a.write(stream,true);
	// Check the stream's contents
	std::string output = stream.str();
	std::vector<std::string> lines;
	boost::split(lines,output,boost::is_any_of("\n"));
	BOOST_CHECK_EQUAL(lines.size(),5u);
	BOOST_CHECK_EQUAL(lines[0],"three\ta\tb");
	BOOST_CHECK_EQUAL(lines[1],"0\t1\t2");
	BOOST_CHECK_EQUAL(lines[2],"1\t7373\t2");
	BOOST_CHECK_EQUAL(lines[3],"2\t1\t2");
}


BOOST_AUTO_TEST_SUITE_END()
