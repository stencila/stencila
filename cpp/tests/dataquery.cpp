/*
Copyright (c) 2012, Stencila Ltd
Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#include <tuple>

#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/test.hpp>
#include <stencila/datatable.hpp>
#include <stencila/dataquery.hpp>
#include <stencila/dataquery-cxx.hpp>
using namespace Stencila;

struct dataqueryFixture { 
	Datatable data;
	
	Column year;
	Column month;
	Column sales;
	
	dataqueryFixture(void):
		year("year"),
		month("month"),
		sales("sales")
	{
		data.name("data");
		data.add("year",Integer);
		data.add("month",Integer);
		data.add("sales",Real);
	} 
	
	void dql_check(Dataquery query, const std::string& sql_, const std::string& dql="") { 
		query.from("data");
		
		std::string sql = query.sql();
		BOOST_CHECK_EQUAL(sql, sql_);
		
		data.dataset().execute(sql);
		
		if(dql.length()>0){
			BOOST_CHECK_EQUAL(query.dql(), dql);
		}
	}
   
};

BOOST_FIXTURE_TEST_SUITE(dataquery,dataqueryFixture)

BOOST_AUTO_TEST_CASE(directives){
	Constant<int> _42(42) ;
	LessThan sales_lessthan_42(sales,_42);
}

BOOST_AUTO_TEST_CASE(dql){
    using namespace DQL;
    
	dql_check(
		Dataquery(),
		"SELECT * FROM \"data\"",
		"data[]"
	);
	
	dql_check(
		Dataquery(sales),
		"SELECT \"sales\" FROM \"data\"",
		"data[sales]"
	);
		
	dql_check(
		Dataquery(sales,year,month),
		"SELECT \"sales\", \"year\", \"month\" FROM \"data\"",
		"data[sales,year,month]"
	);
	dql_check(
		Dataquery(distinct),
		"SELECT DISTINCT * FROM \"data\""
	);
	dql_check(
		Dataquery(all),
		"SELECT * FROM \"data\""
	);
	dql_check(
		Dataquery(distinct,all),
		"SELECT * FROM \"data\""
	);
	
	dql_check(
		Dataquery(where(1)),
		"SELECT * FROM \"data\" WHERE 1",
		"data[where(1)]"
	);
	dql_check(
		Dataquery(where(sales<10)),
		"SELECT * FROM \"data\" WHERE \"sales\"<10",
		"data[where(sales<10)]"
	);
	dql_check(
		Dataquery(where((month<=10 or sales>10) and sales>100)),
		R"(SELECT * FROM "data" WHERE (("month"<=10) OR ("sales">10)) AND ("sales">100))",
		"data[where(((month<=10) or (sales>10)) and (sales>100))]"
	);
	dql_check(
		Dataquery(where(year+month+10>sales+10)),
		R"(SELECT * FROM "data" WHERE (("year"+"month")+10)>("sales"+10))",
		"data[where(((year+month)+10)>(sales+10))]"
	);
	dql_check(
		Dataquery(by(year),sum(sales)),
		R"(SELECT "year", sum("sales") FROM "data" GROUP BY "year")",
		"data[by(year),sum(sales)]"
	);
	dql_check(
		Dataquery(by(year),by(month),max(sales)),
		R"(SELECT "year", "month", max("sales") FROM "data" GROUP BY "year", "month")",
		"data[by(year),by(month),max(sales)]"
	);
	
	dql_check(
		Dataquery(by(year),having(sum(sales)>1000)),
		"SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING sum(\"sales\")>1000",
		"data[by(year),having(sum(sales)>1000)]"
	);
	dql_check(
		Dataquery(by(year),having(sum(sales)>1000 and year<2000)),
		"SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING (sum(\"sales\")>1000) AND (\"year\"<2000)",
		"data[by(year),having((sum(sales)>1000) and (year<2000))]"
	);
	dql_check(
		Dataquery(by(year),having(sum(sales)>1000),having(year<2000)),
		"SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING (sum(\"sales\")>1000) AND (\"year\"<2000)",
		"data[by(year),having(sum(sales)>1000),having(year<2000)]"
	);
	
	dql_check(
		Dataquery(order(year),order(sales,-1)),
		"SELECT * FROM \"data\" ORDER BY \"year\" ASC, \"sales\" DESC",
		"data[order(year),order(sales,-1)]"
	);
	dql_check(
		Dataquery(by(year),by(month),order(max(sales),-1)),
		"SELECT \"year\", \"month\" FROM \"data\" GROUP BY \"year\", \"month\" ORDER BY max(\"sales\") DESC"
	);

	dql_check(
		Dataquery(limit(10)),
		"SELECT * FROM \"data\" LIMIT 10"
	);
	
	dql_check(
		Dataquery(offset(10)),
		"SELECT * FROM \"data\" LIMIT 9223372036854775807 OFFSET 10"
	);
	
	dql_check(
		Dataquery(by(year),by(month),sum(sales),where(month>6 and year>2000),having(sum(sales)>1000),offset(10),limit(1000)), 
		"SELECT \"year\", \"month\", sum(\"sales\") FROM \"data\" WHERE (\"month\">6) AND (\"year\">2000) GROUP BY \"year\", \"month\" HAVING sum(\"sales\")>1000 LIMIT 1000 OFFSET 10"
	);
} 

BOOST_AUTO_TEST_SUITE_END()
