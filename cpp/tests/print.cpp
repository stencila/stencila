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

#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <array>
#include <vector>
#include <map>
#include <set>

#include <stencila/print.hpp>

BOOST_AUTO_TEST_SUITE(printing)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(cout){
    //! @class Printer<void>
    //! Test that print>>"Hello World!" prints to std::cout
    
    //Redirect std::cout to a new string stream
    std::streambuf* stdcout = std::cout.rdbuf();
    std::stringstream output;
    std::cout.rdbuf(output.rdbuf());
    //Do test
    print<<"Hello World!";
    BOOST_CHECK_EQUAL(output.str(),"Hello World!");
    // Redirect std::cout to its old buffer
    std::cout.rdbuf(stdcout);
}

BOOST_AUTO_TEST_CASE(ostringstream){
    //! @class Printer<std::ostringstream>
    //! Test that print()>>"Hello World!" prints to a new std::string

	std::string output = print()<<"Hello World!";
	BOOST_CHECK_EQUAL(output,"Hello World!");
}

BOOST_AUTO_TEST_CASE(ostream){
	//! @class Printer<std::ofstream>
    //! Test that print(stream)<<"Hello world!" prints to an existing output stream
	std::ostringstream output;
	print(output)<<"Hello World!";
    BOOST_CHECK_EQUAL(output.str(),"Hello World!");
}

BOOST_AUTO_TEST_CASE(ofstream){
    //! @class Printer<std::ofstream>
    //! Test that print("filename")>>"Hello World!" prints to a new file

    const char* filename = boost::filesystem::unique_path().native().c_str();
    print(filename)<<"Hello World!"<<$$;
    //Read in output
    std::ifstream file(filename);
    std::string output;
    std::getline(file,output);
	BOOST_CHECK_EQUAL(output,"Hello World!");
}


BOOST_AUTO_TEST_CASE(printing){
	//! @class Printer<void>
    //! Test formatting
		
    // A local macro to test alternative versions of print method
	#define CHECK(expr,str) BOOST_CHECK_EQUAL(Printer<void>::print(expr),str);
	
	CHECK(42,"42")
    int _42 = 42;
    CHECK(&_42,"&42")
    
    CHECK(3.14,"3.14")
    float _pi = 3.14;
    CHECK(&_pi,"&3.14")
    
    CHECK((std::make_pair("foo",3.14)),"(\"foo\",3.14)")
    CHECK((std::make_tuple("foo",3.14,'a')),"(\"foo\",3.14,'a')")
    
    CHECK((std::array<int,3>{1,2,3}),"[1,2,3]")
    CHECK((std::vector<int>{1,2,3}),"[1,2,3]")
	CHECK((std::vector<std::string>{"foo","bar"}),"[\"foo\",\"bar\"]")
	CHECK((std::vector<std::vector<int>>{{1,2,3},{4,5,6},{7,8,9}}),"[[1,2,3],[4,5,6],[7,8,9]]")
    CHECK((std::vector<int*>{&_42,0,&_42}),"[&42,&null,&42]")
    
    CHECK((std::map<int,std::string>{{1,"a"},{2,"b"}}),"{1:\"a\",2:\"b\"}")
    //CHECK((std::set<int>{1,2,3}),"{1,2,3}")
		
	#undef CHECK
}

BOOST_AUTO_TEST_SUITE_END()
