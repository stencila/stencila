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

BOOST_AUTO_TEST_SUITE(traits)

using namespace Stencila::Traits;

BOOST_AUTO_TEST_CASE(cout){
    typedef std::vector<int> vec;
	BOOST_CHECK_EQUAL(IsContainer<vec>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<vec>::value,false);
    BOOST_CHECK_EQUAL(IsPaired<vec>::value,false);
    
    typedef std::set<int> set;
	BOOST_CHECK_EQUAL(IsContainer<set>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<set>::value,true);
    BOOST_CHECK_EQUAL(IsPaired<set>::value,false);
    
    typedef std::map<int,int> map;
	BOOST_CHECK_EQUAL(IsContainer<map>::value,true);
	BOOST_CHECK_EQUAL(IsAssociative<map>::value,true);
    BOOST_CHECK_EQUAL(IsPaired<map>::value,true);
}

BOOST_AUTO_TEST_SUITE_END()
