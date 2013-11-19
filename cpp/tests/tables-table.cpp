#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/test.hpp>
#include <stencila/tables/table.hpp>
using namespace Stencila::Tables;

struct tableFixture {
	Tableset tableset;
	
	tableFixture(){
		tableset.execute(
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

BOOST_FIXTURE_TEST_SUITE(tableTests,tableFixture)

BOOST_AUTO_TEST_CASE(constructors){
	//! @class Stencila:Datatable
	//! @test Test constructors
	Table t1 = tableset.table("t1");
	BOOST_CHECK_EQUAL(&t1.tableset(),&tableset);
	BOOST_CHECK_EQUAL(t1.name(),"t1");
}

BOOST_AUTO_TEST_CASE(attributes){
	//! @class Stencila:Datatable
	//! @test Test attributes (e.g. rows, columns, names etc)
	Table t1 = tableset.table("t1");
	
	BOOST_CHECK_EQUAL(t1.rows(),(unsigned int)5);
	BOOST_CHECK_EQUAL(t1.columns(),(unsigned int)3);
	
	std::vector<unsigned int> dims = t1.dimensions();
	BOOST_CHECK_EQUAL(dims.size(),(unsigned int)2);
	BOOST_CHECK_EQUAL(dims[0],t1.rows());
	BOOST_CHECK_EQUAL(dims[1],t1.columns());
	
	BOOST_CHECK_EQUAL(t1.name(0),"c1");
	BOOST_CHECK_EQUAL(t1.name(1),"c2");
	BOOST_CHECK_EQUAL(t1.name(2),"c3");

	check_equal(t1.names(),std::vector<std::string>{"c1","c2","c3"});
		
	// Require operator<< to be defined for Datatype, so commented out for now...
	//BOOST_CHECK_EQUAL(t1.type(0),Integer);
	//BOOST_CHECK_EQUAL(t1.type(1),Real);
	//BOOST_CHECK_EQUAL(t1.type(2),Text);
	//check_equal(t1.types(),std::vector<Datatype>{Integer,Real,Text});
	
	check_equal(t1.indices(),std::vector<std::string>{"t1_c1"});
}

BOOST_AUTO_TEST_CASE(sql){
	//! @class Stencila:Datatable
	//! @test Test the execution of SQL
	Table t1 = tableset.table("t1");
	t1.execute("INSERT INTO t1 VALUES(6,6.6,'zeta')");
	check_equal(
		t1.cursor("SELECT * FROM t1 ORDER BY c1 DESC LIMIT 1;").row<std::vector<std::string>>(),
		std::vector<std::string>{"6","6.6","zeta"}
	);
	BOOST_CHECK_EQUAL(
		t1.fetch("SELECT * FROM t1 WHERE c1<=2;").size(),
		(unsigned int)2
	);	
	BOOST_CHECK_EQUAL(
		t1.fetch("SELECT * FROM t1 WHERE c1>900;").size(),
		(unsigned int)0
	);	
}

BOOST_AUTO_TEST_SUITE_END()
 