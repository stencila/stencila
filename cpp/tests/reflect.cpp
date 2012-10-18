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
#include <string>

#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/test.hpp>
#include <stencila/print.hpp>
#include <stencila/reflect.hpp>

BOOST_AUTO_TEST_SUITE(reflect)

using namespace Stencila;
using namespace Stencila::Reflect;

class Person : public Reflector<Person> {
public:
	std::string name;
	int age;
    
    Person(std::string name_="", int age_=0):
        name(name_),
        age(age_){
    }
	
	std::string greet(void) const {
		return print()<<"Hello, my name is "<<name<<", I am "<<age<<" years old.";
	}

	template<class Reflection>
	void reflect(Reflection& r){
		r.data("name",&name)
		 .data("age",&age)
		 .method("greet",&Person::greet);
	}
};
Register<Person> Person_;


class Couple : public Reflector<Couple> {
public:
	Person a;
	Person b;	

	REFLECT(
		DATA(a)
		DATA(b)
	)
};
Register<Couple> Couple_;


BOOST_AUTO_TEST_CASE(registry){
    //print<<Registry.types();
}

BOOST_AUTO_TEST_CASE(is_reflector){
    BOOST_CHECK_EQUAL(IsReflector<float>::value,false);
    BOOST_CHECK_EQUAL(IsReflector<Person>::value,true);
}

BOOST_AUTO_TEST_CASE(type_introspection_fund){
	bool v = false;
    std::string type = Type().mirror(v).type();
    BOOST_CHECK_EQUAL(type,"bool");
    
    std::vector<std::string> keys = Keys().mirror(v).keys();
    BOOST_CHECK_EQUAL(keys.size(),0);
}

BOOST_AUTO_TEST_CASE(type_introspection){
	std::string type = Type().mirror<Person>().type();
    BOOST_CHECK_EQUAL(type,"reflect::Person");
    
    std::vector<std::string> keys = Keys().mirror<Person>().keys();
    check_equal(keys,std::vector<std::string>{"name","age","greet"});
    
	bool has = Has("name").mirror<Person>().has();
    BOOST_CHECK(has);
}

BOOST_AUTO_TEST_CASE(type_introspection_macros){
	std::string type = Type().mirror<Couple>().type();
    BOOST_CHECK_EQUAL(type,"reflect::Couple");
    
    std::vector<std::string> keys = Keys().mirror<Couple>().keys();
    check_equal(keys,std::vector<std::string>{"a","b"});
    
	bool has = Has("a").mirror<Couple>().has();
    BOOST_CHECK(has);
}

BOOST_AUTO_TEST_CASE(instance_introspection){
	Person john("John",29);
    
	BOOST_CHECK_EQUAL(john.type(),"reflect::Person");
    
    auto keys = john.keys();
    check_equal(keys,std::vector<std::string>{"name","age","greet"});
    
    BOOST_CHECK(john.has("name"));
    BOOST_CHECK(john.has("age"));
    BOOST_CHECK(john.has("greet"));
    
    auto name = john.get("name");
    BOOST_CHECK_EQUAL(name.type(),"std::string");
    BOOST_CHECK(name.object()!=0);
    BOOST_CHECK_EQUAL(*static_cast<std::string*>(name.object()),"John");
    
    auto age = john.get("age");
    BOOST_CHECK_EQUAL(age.type(),"int");
    BOOST_CHECK(age.object()!=0);
    BOOST_CHECK_EQUAL(*static_cast<unsigned short int*>(age.object()),29);
}

BOOST_AUTO_TEST_CASE(instance_introspection_dynamic){
	auto john = Create("reflect::Person");
    
    BOOST_CHECK_EQUAL(john.type(),"reflect::Person");
    
    check_equal(john.keys(),std::vector<std::string>{"name","age","greet"});
    
    auto name = john["name"];
    //print<<name.type()<<" "<<name.keys()<<" "<<name.has("p");
}

BOOST_AUTO_TEST_SUITE_END()
