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

#include <stencila/testing.hpp>
#include <stencila/datatable.hpp>
#include <stencila/dataquery.hpp>
#include <stencila/eql.hpp>
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
	
	void eql_check(Dataquery query, const std::string& sql_, const std::string& dql="") { 
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

 
BOOST_AUTO_TEST_CASE(eql){
	using namespace EQL;
	
	eql_check(
		get(),
		"SELECT * FROM \"data\"",
		"data[]"
	);
	
	eql_check(
		get(sales),
		"SELECT \"sales\" FROM \"data\"",
		"data[sales]"
	);
		
	eql_check(
		get(sales,year,month),
		"SELECT \"sales\", \"year\", \"month\" FROM \"data\"",
		"data[sales,year,month]"
	);
	eql_check(
		get(distinct),
		"SELECT DISTINCT * FROM \"data\""
	);
	eql_check(
		get(all),
		"SELECT * FROM \"data\""
	);
	eql_check(
		get(distinct,all),
		"SELECT * FROM \"data\""
	);
	
	eql_check(
		get(where(1)),
		"SELECT * FROM \"data\" WHERE 1",
		"data[where(1)]"
	);
	eql_check(
		get(where(sales<10)),
		"SELECT * FROM \"data\" WHERE \"sales\"<10",
		"data[where(sales<10)]"
	);
	eql_check(
		get(where(_(month<=10 or sales>10) and sales>100)),
		"SELECT * FROM \"data\" WHERE (\"month\"<=10 OR \"sales\">10) AND \"sales\">100",
		"data[where((month<=10 or sales>10) and sales>100)]"
	);
	eql_check(
		get(where(year+month+10>sales+10)),
		"SELECT * FROM \"data\" WHERE \"year\"+\"month\"+10>\"sales\"+10",
		"data[where(year+month+10>sales+10)]"
	);
	eql_check(
		get(by(year),sum(sales)),
		"SELECT \"year\", sum(\"sales\") FROM \"data\" GROUP BY \"year\"",
		"data[by(year),sum(sales)]"
	);
	eql_check(
		get(by(year),by(month),max(sales)),
		"SELECT \"year\", \"month\", max(\"sales\") FROM \"data\" GROUP BY \"year\", \"month\"",
		"data[by(year),by(month),max(sales)]"
	);
	
	eql_check(
		get(by(year),having(sum(sales)>1000)),
		"SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING sum(\"sales\")>1000",
		"data[by(year),having(sum(sales)>1000)]"
	);
	eql_check(
		get(by(year),having(sum(sales)>1000 and year<2000)),
		"SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING sum(\"sales\")>1000 AND \"year\"<2000",
		"data[by(year),having(sum(sales)>1000 and year<2000)]"
	);
	eql_check(
		get(by(year),having(sum(sales)>1000),having(year<2000)),
		"SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING (sum(\"sales\")>1000) AND (\"year\"<2000)",
		"data[by(year),having(sum(sales)>1000),having(year<2000)]"
	);
	
	eql_check(
		get(order(year),order(sales,-1)),
		"SELECT * FROM \"data\" ORDER BY \"year\" ASC, \"sales\" DESC",
		"data[order(year),order(sales,-1)]"
	);
	eql_check(
		get(by(year),by(month),order(max(sales),-1)),
		"SELECT \"year\", \"month\" FROM \"data\" GROUP BY \"year\", \"month\" ORDER BY max(\"sales\") DESC"
	);

	eql_check(
		get(limit(10)),
		"SELECT * FROM \"data\" LIMIT 10"
	);
	
	eql_check(
		get(offset(10)),
		"SELECT * FROM \"data\" LIMIT 9223372036854775807 OFFSET 10"
	);
	
	eql_check(
		get(by(year),by(month),sum(sales),where(month>6 and year>2000),having(sum(sales)>1000),offset(10),limit(1000)), 
		"SELECT \"year\", \"month\", sum(\"sales\") FROM \"data\" WHERE \"month\">6 AND \"year\">2000 GROUP BY \"year\", \"month\" HAVING sum(\"sales\")>1000 LIMIT 1000 OFFSET 10"
	);
} 

BOOST_AUTO_TEST_SUITE_END()
