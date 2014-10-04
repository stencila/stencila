#include <boost/test/unit_test.hpp>

#include <array>
#include <map>
#include <set>
#include <vector>

#include <stencila/traits.hpp>
#include <stencila/reflector.hpp>

BOOST_AUTO_TEST_SUITE(traits)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(traits){

	auto lambda1 = [](){};
	BOOST_CHECK((std::is_same<FunctionTraits<decltype(lambda1)>::return_type,void>::value));

	auto lambda2 = [](char a, int b, std::string c){return double(0);};
	BOOST_CHECK((std::is_same<FunctionTraits<decltype(lambda2)>::return_type,double>::value));
	BOOST_CHECK_EQUAL((FunctionTraits<decltype(lambda2)>::arity),3);
	BOOST_CHECK((std::is_same<FunctionTraits<decltype(lambda2)>::args<0>::type,char>::value));
	BOOST_CHECK((std::is_same<FunctionTraits<decltype(lambda2)>::args<1>::type,int>::value));
	BOOST_CHECK((std::is_same<FunctionTraits<decltype(lambda2)>::args<2>::type,std::string>::value));
	

	struct Functor{
		void operator()(){}
	};
	BOOST_CHECK_EQUAL(IsCallable<Functor>::value,true);
	auto lambda10 = [](){};
	BOOST_CHECK_EQUAL(IsCallable<decltype(lambda10)>::value,true);
	BOOST_CHECK_EQUAL(IsCallable<double>::value,false);


    typedef std::vector<int> vec;
	BOOST_CHECK_EQUAL(IsContainer<vec>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<vec>::value,false);
    BOOST_CHECK_EQUAL(IsPaired<vec>::value,false);

    typedef std::array<double,10> arr;
	BOOST_CHECK_EQUAL(IsContainer<arr>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<arr>::value,false);
    BOOST_CHECK_EQUAL(IsPaired<arr>::value,false);
    
    typedef std::set<int> set;
	BOOST_CHECK_EQUAL(IsContainer<set>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<set>::value,true);
    BOOST_CHECK_EQUAL(IsPaired<set>::value,false);
    
    typedef std::map<int,int> map;
	BOOST_CHECK_EQUAL(IsContainer<map>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<map>::value,true);
    BOOST_CHECK_EQUAL(IsPaired<map>::value,true);
}

BOOST_AUTO_TEST_CASE(reflector){
    struct A : Reflector<A> {};
	BOOST_CHECK_EQUAL(HasReflect<A>::value,true);
	BOOST_CHECK_EQUAL(IsReflector<A>::value,true);
}

BOOST_AUTO_TEST_SUITE_END()
