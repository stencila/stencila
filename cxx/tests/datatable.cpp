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
#include "../datatable.hpp"
using namespace Stencila;

struct datatableFixture {
	Dataset dataset;
	
	datatableFixture(){
		dataset.execute(
			"CREATE TABLE t1 ("
			"	c1 INTEGER,"
			"	c2 REAL,"
			"	c3 TEXT"
			");"
		
			"INSERT INTO t1 VALUES(1,1.1,'alpha');"
			"INSERT INTO t1 VALUES(2,2.2,'beta');"
			"INSERT INTO t1 VALUES(3,3.3,'gamma');"
			"INSERT INTO t1 VALUES(4,4.4,'delta');"
			"INSERT INTO t1 VALUES(5,5.5,'epsilon');"
		
			"CREATE INDEX t1_c1 ON t1(c1);"
		);
	}
};

BOOST_FIXTURE_TEST_SUITE(datatable,datatableFixture)

BOOST_AUTO_TEST_CASE(constructors){
	//! @class Stencila:Datatable
	//! @test Test constructors
	Datatable t1 = dataset.table("t1");
	check_equal(&t1.dataset(),&dataset);
	check_equal(t1.name(),"t1");
}

BOOST_AUTO_TEST_CASE(attributes){
	//! @class Stencila:Datatable
	//! @test Test attributes (e.g. rows, columns, names etc)
	Datatable t1 = dataset.table("t1");
	
	check_equal(t1.rows(),5);
	check_equal(t1.columns(),3);
	
	std::vector<unsigned int> dims = t1.dimensions();
	check_equal(dims.size(),2);
	check_equal(dims[0],t1.rows());
	check_equal(dims[1],t1.columns());
	
	check_equal(t1.name(0),"c1");
	check_equal(t1.name(1),"c2");
	check_equal(t1.name(2),"c3");
	check_equal(t1.names(),std::vector<std::string>{"c1","c2","c3"});
	
	check_equal(t1.type(0),Integer);
	check_equal(t1.type(1),Real);
	check_equal(t1.type(2),Text);
	check_equal(t1.types(),std::vector<Datatype>{Integer,Real,Text});
	
	check_equal(t1.indices(),std::vector<std::string>{"t1_c1"});
}

BOOST_AUTO_TEST_CASE(sql){
	//! @class Stencila:Datatable
	//! @test Test the execution of SQL
	Datatable t1 = dataset.table("t1");
	t1.execute("INSERT INTO t1 VALUES(6,6.6,'zeta')");
	check_equal(
		t1.query("SELECT * FROM t1 ORDER BY c1 DESC LIMIT 1;").row<std::vector<std::string>>(),
		std::vector<std::string>{"6","6.6","zeta"}
	);
	check_equal(
		t1.fetch("SELECT * FROM t1 WHERE c1<=2;").size(),
		2
	);	
	check_equal(
		t1.fetch("SELECT * FROM t1 WHERE c1>900;").size(),
		0
	);	
}

BOOST_AUTO_TEST_SUITE_END()
 