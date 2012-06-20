/*
Copyright (c) 2012, Nokome Bentley, nokome.bentley@stenci.la

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#define BOOST_TEST_DYN_LINK
#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include "../testing.hpp"
#include "../dataset.hpp"
#include "../datatable.hpp"
using namespace Stencila;

struct datasetFixture {
	Dataset dataset;
	
	datasetFixture(){
		dataset.execute(
			"CREATE TABLE t1(c1 TEXT);"
			"CREATE TABLE t2(c1 TEXT);"
			"CREATE INDEX t1_c1 ON t1(c1);"
			"CREATE INDEX t2_c1 ON t2(c1);"
		);
	}
};

BOOST_FIXTURE_TEST_SUITE(dataset,datasetFixture)

BOOST_AUTO_TEST_CASE(tables){
	check_equal(dataset.tables(),std::vector<std::string>{"t1","t2"});
	
	Datatable table1 = dataset.table("t1");
	check_equal(table1.name(),"t1");
}

BOOST_AUTO_TEST_CASE(indices){
	check_equal(dataset.indices(),std::vector<std::string>{"t1_c1","t2_c1"});
}

BOOST_AUTO_TEST_SUITE_END()