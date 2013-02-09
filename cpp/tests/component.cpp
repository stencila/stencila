/*
Copyright (c) 2013, Stencila Ltd
Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#include <iostream>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/component.hpp>

BOOST_AUTO_TEST_SUITE(component)

using namespace Stencila;

class Test : public Component<Test> {};

BOOST_AUTO_TEST_CASE(construct){
    Test c1, c2;
    BOOST_CHECK_EQUAL(c1.id().length(),32);
    BOOST_CHECK(c1.id()!=c2.id());
    BOOST_CHECK_EQUAL(Test::obtain<Test>(c1.id()),&c1);
    BOOST_CHECK_EQUAL(Test::filter<Test>().size(),2);
}

BOOST_AUTO_TEST_SUITE_END()

template<>
const char* Stencila::Component<component::Test>::type = "test";
