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

#define BOOST_TEST_DYN_LINK
#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include "../testing.hpp"
#include "../dataquery.hpp"
#include "../datatable.hpp"
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(dataquery)
    
BOOST_AUTO_TEST_CASE(sql){
	
	Datatable data("data");
	data.add("year",Integer);
	data.add("month",Integer);
	data.add("sales",Real);
	
	std::vector<
		std::pair<Dataquery,std::string>
	> tests = {
		{get(),"SELECT * FROM data"},
		{get("sales"),"SELECT sales FROM data"},
		{get("sales","year"),"SELECT sales, year FROM data"},
		
		{get().columns("sales"),"SELECT sales FROM data"},
		{get().columns("sales","year"),"SELECT sales, year FROM data"},
		
		{get().distinct(),"SELECT DISTINCT * FROM data"},
		{get().distinct(false),"SELECT * FROM data"},
		{get().all(),"SELECT * FROM data"},
		{get().all(false),"SELECT DISTINCT * FROM data"},
		{get().distinct().all(),"SELECT * FROM data"},
		
		{get().where("1"),"SELECT * FROM data WHERE 1"},
		{get().where("sales>10"),"SELECT * FROM data WHERE sales>10"},
		{get().where("1","sales>10"),"SELECT * FROM data WHERE (1) AND (sales>10)"},
		
		{get("sum(sales)").by("year"),"SELECT year, sum(sales) FROM data GROUP BY year"},
		{get("sum(sales)").by("year").by("month"),"SELECT year, month, sum(sales) FROM data GROUP BY year, month"},
		{get("sum(sales)").by("year","month"),"SELECT year, month, sum(sales) FROM data GROUP BY year, month"},
		
		{get().by("year").having("sum(sales)>1000"),"SELECT year FROM data GROUP BY year HAVING sum(sales)>1000"},
		{get().by("year").having("sum(sales)>1000 AND year<2000"),"SELECT year FROM data GROUP BY year HAVING sum(sales)>1000 AND year<2000"},
		{get().by("year").having("sum(sales)>1000","year<2000"),"SELECT year FROM data GROUP BY year HAVING (sum(sales)>1000) AND (year<2000)"},
		{get().by("year").having("sum(sales)>1000").having("year<2000"),"SELECT year FROM data GROUP BY year HAVING (sum(sales)>1000) AND (year<2000)"},

		{get().limit(10), "SELECT * FROM data LIMIT 10"},
		
		{get().offset(10), "SELECT * FROM data LIMIT 9223372036854775807 OFFSET 10"},
		
		{
			get("sum(sales)").by("year","month").where("month>6","year>2000").having("sum(sales)>1000").offset(10).limit(1000), 
			"SELECT year, month, sum(sales) FROM data WHERE (month>6) AND (year>2000) GROUP BY year, month HAVING sum(sales)>1000 LIMIT 1000 OFFSET 10"
		},
		
	};
	for(auto test=tests.begin();test!=tests.end();test++){
		const Dataquery& query = test->first;
		std::string sql = query.sql("data");
		BOOST_CHECK_EQUAL(sql, test->second);
		data.dataset().execute(sql);
	}
} 

BOOST_AUTO_TEST_SUITE_END()
