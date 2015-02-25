#include <boost/test/unit_test.hpp>

#include <stencila/frame.hpp>
#include <stencila/array.hpp>
#include <stencila/structure.hpp>

BOOST_AUTO_TEST_SUITE(frame_quick)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(basic){
	Frame frame;

	BOOST_CHECK_EQUAL(frame.rows(),0u);
	BOOST_CHECK_EQUAL(frame.columns(),0u);
	BOOST_CHECK_EQUAL(frame.labels().size(),0u);

	frame.add("col1");

	BOOST_CHECK_EQUAL(frame.rows(),0u);
	BOOST_CHECK_EQUAL(frame.columns(),1u);
	BOOST_CHECK_EQUAL(frame.labels().size(),1u);
	BOOST_CHECK_EQUAL(frame.label(0),"col1");

	frame.append();
	BOOST_CHECK_EQUAL(frame.rows(),1u);
	BOOST_CHECK_EQUAL(frame.columns(),1u);

	frame(0,0) = 42;
	BOOST_CHECK_EQUAL(frame(0,0),42);

	frame(0,0) = 3.14;
	BOOST_CHECK_EQUAL(frame(0,0),3.14);
	
}

BOOST_AUTO_TEST_CASE(construct){
	Frame frame1;
	BOOST_CHECK_EQUAL(frame1.rows(),0u);
	BOOST_CHECK_EQUAL(frame1.columns(),0u);

	Frame frame2({"a","b","c"},100);
	BOOST_CHECK_EQUAL(frame2.rows(),100u);
	BOOST_CHECK_EQUAL(frame2.columns(),3u);
	BOOST_CHECK(frame2.has("b"));
	BOOST_CHECK(not frame2.has("p"));

	Frame frame3(100,{"a","b","c"});
	BOOST_CHECK_EQUAL(frame3.rows(),100u);
	BOOST_CHECK_EQUAL(frame3.columns(),3u);
	BOOST_CHECK_EQUAL(frame3.label(1),"b");
	frame3(0,0) = 1.2;
	frame3(20,2) = 1.3;

	Frame frame4 = frame3;
	BOOST_CHECK_EQUAL(frame4.rows(),100u);
	BOOST_CHECK_EQUAL(frame4.columns(),3u);
	BOOST_CHECK_EQUAL(frame4.label(1),"b");
	BOOST_CHECK_EQUAL(frame4(0,0),1.2);
	BOOST_CHECK_EQUAL(frame4(20,2),1.3);
}


BOOST_AUTO_TEST_CASE(append){
	Frame frame1({"a","b","c"},10);

	Frame frame2 = frame1;
	frame2(0,0) = 0.1;
	frame2(0,1) = 0.2;
	frame2(0,2) = 0.3;
	frame2(1,0) = 1.1;
	frame2(1,1) = 1.2;
	frame2(1,2) = 1.3;

	frame1.append();
	BOOST_CHECK_EQUAL(frame1.rows(),11u);

	frame1.append(3);
	BOOST_CHECK_EQUAL(frame1.rows(),14u);

	frame1.append(frame2);
	BOOST_CHECK_EQUAL(frame1.rows(),24u);
	BOOST_CHECK_EQUAL(frame1(14,1),0.2);
	BOOST_CHECK_EQUAL(frame1(15,2),1.3);
}

BOOST_AUTO_TEST_CASE(slice){
	Frame frame(
		{"a","b"},
		{
			1.1,1.2,
			2.1,2.2
		}
	);
	Frame slice = frame.slice(1);
	BOOST_CHECK_EQUAL(slice.rows(),1u);
	BOOST_CHECK_EQUAL(slice.columns(),2u);
	BOOST_CHECK_EQUAL(slice(0,0),2.1);
	BOOST_CHECK_EQUAL(slice(0,1),2.2);
}

STENCILA_DIM(Two,two,two,2);

struct A : public Structure<A> {

	bool a = true;
	char b  = 'b';
	Array<int,Two> c;

	template<class Mirror>
	void reflect(Mirror& mirror){
		mirror
			.data(a,"a")
			.data(b,"b")
			.data(c,"c")
		;
	}
};

BOOST_AUTO_TEST_CASE(from_structure){
	Frame frame = Frame::of<A>();

	BOOST_CHECK_EQUAL(frame.rows(),0u);
	BOOST_CHECK_EQUAL(frame.columns(),4u);

	auto labels = frame.labels();
	BOOST_CHECK_EQUAL(labels[0],"a");
	BOOST_CHECK_EQUAL(labels[1],"b");
	BOOST_CHECK_EQUAL(labels[2],"c(0)");
	BOOST_CHECK_EQUAL(labels[3],"c(1)");
}

BOOST_AUTO_TEST_SUITE_END()
