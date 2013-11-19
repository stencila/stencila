#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/test.hpp>
#include <stencila/tables/tableset.hpp>
#include <stencila/tables/table.hpp>

using namespace Stencila::Tables;

struct datasetFixture { 
	Tableset tableset;
	
	datasetFixture(){
		tableset.execute(
			//Caution: do not put empty lines in this SQL otherwise it breaks it 
			//into several strings of which only the first is passed as an argument!
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
			"CREATE TABLE t2(c1 TEXT);"
			"CREATE INDEX t1_c1 ON t1(c1);"
			"CREATE INDEX t2_c1 ON t2(c1);"
		);
		tableset.import("t1");
		tableset.import("t2");
	} 
};

BOOST_FIXTURE_TEST_SUITE(tableset,datasetFixture)

BOOST_AUTO_TEST_CASE(cursor){ 
	auto cursor = tableset.cursor("SELECT max(c1) FROM t1");
	BOOST_CHECK_EQUAL(cursor.value<int>(),5);
}

BOOST_AUTO_TEST_CASE(tables){ 
	auto tables = tableset.tables();

	BOOST_CHECK_EQUAL(tables.size(),(unsigned int)2);

	BOOST_CHECK_EQUAL(tables[0],"t1");
	BOOST_CHECK_EQUAL(tables[1],"t2");

	BOOST_CHECK_EQUAL(tableset.exists("table","t1"),true);
	BOOST_CHECK_EQUAL(tableset.exists("table","foo"),false);

	Table table1 = tableset.table("t1"); 
	BOOST_CHECK_EQUAL(table1.name(),"t1");
}

BOOST_AUTO_TEST_CASE(indices){
	auto indices = tableset.indices();
	BOOST_CHECK_EQUAL(indices.size(),(unsigned int)2);
	BOOST_CHECK_EQUAL(indices[0],"t1_c1");
	BOOST_CHECK_EQUAL(indices[1],"t2_c1");
}

BOOST_AUTO_TEST_CASE(caching){
    tableset.select("SELECT max(c2) FROM t1");
    std::string sql = "SELECT sum(c2) FROM t1";
    tableset.select(sql);
    BOOST_CHECK_EQUAL(tableset.cached(),2); 
    BOOST_CHECK_EQUAL(tableset.cached(sql),1);
    tableset.select(sql);

    //Save a copy of the tableset and make sure that
    //the copy has the right cached number
    tableset.backup("outputs/tableset-caching.sted");
    Tableset dataset_copy("outputs/tableset-caching.sted");
    BOOST_CHECK_EQUAL(dataset_copy.cached(),2);

    //Vacuum the tableset
    tableset.vacuum();
    BOOST_CHECK_EQUAL(tableset.cached(),0);
    BOOST_CHECK_EQUAL(tableset.cached(sql),0);
}

BOOST_AUTO_TEST_CASE(functions){
}

BOOST_AUTO_TEST_CASE(aggregators){
    BOOST_CHECK_CLOSE(tableset.value<float>("SELECT mean(c2) FROM t1"),3.3,0.0001); //mean(c2)
    BOOST_CHECK_CLOSE(tableset.value<float>("SELECT geomean(c2) FROM t1"),2.865688,0.0001); //exp(mean(log(c2)))
    BOOST_CHECK_CLOSE(tableset.value<float>("SELECT harmean(c2) FROM t1"),2.408759,0.0001); //length(c2)/sum(1/c2)
    
    BOOST_CHECK_CLOSE(tableset.value<float>("SELECT var(c2) FROM t1"),3.025,0.0001); //var(c2)
    BOOST_CHECK_CLOSE(tableset.value<float>("SELECT sd(c2) FROM t1"),1.739253,0.0001); //sd(c2)
}

BOOST_AUTO_TEST_CASE(aggregators_2step){
    Table first = tableset.select("SELECT mean1(c2) AS mean1_ FROM t1");
    BOOST_CHECK_CLOSE(first.value<float>("mean2(mean1_)"),3.3,0.0001);
}

BOOST_AUTO_TEST_SUITE_END()

