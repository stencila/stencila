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

#include <vector>
#include <tuple>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/test.hpp>
#include <stencila/dataquery.hpp>
#include <stencila/dataquery-cxx.hpp>
#include <stencila/dataset.hpp>
#include <stencila/print.hpp>
using namespace Stencila;

struct dataqueryFixture { 
    Datatable data;

    dataqueryFixture(void):
        data("data",
            "year",Integer,
            "month",Integer,
            "sales",Real
        )
    {
        for(int year=2000;year<=2012;year++){
            for(int month=1;month<=12;month++){
                data.append(std::vector<int>{year,month,year*100+month});
            }
        }
    } 
    
    void dql_check(Dataquery query, const std::string& sql_, const std::string& dql="") { 
        
        //! @todo Create another way of testing SQL - perhaps by running SQL
        //! as a separate query
        //std::string sql = query.sql(data);
        //BOOST_CHECK_EQUAL(sql, sql_);
        
        //data.dataset().execute(sql);
        
        if(dql.length()>0){
            BOOST_CHECK_EQUAL(query.dql(), dql);
        }
    }
   
};

BOOST_FIXTURE_TEST_SUITE(dataquery,dataqueryFixture)

BOOST_AUTO_TEST_CASE(directives){
    Constant<int>* _42 = new Constant<int>(42) ;
    Column* sales = new Column("sales");
    LessThan sales_lessthan_42(sales,_42);
}

BOOST_AUTO_TEST_CASE(dql){
    using namespace DQL;
    
    Column year = column("year");
    Column month = column("month");
    Column sales = column("sales");
    
    dql_check(
        query(),
        "SELECT * FROM \"data\"",
        ""
    );
    
    dql_check(
        query(sales),
        "SELECT \"sales\" FROM \"data\"",
        "sales"
    );
        
    dql_check(
        query(as("Sales",sales)),
        "SELECT \"sales\" AS \"Sales\" FROM \"data\"",
        "as(\"Sales\",sales)"
    );

    dql_check(
        query(sales,year,month),
        "SELECT \"sales\", \"year\", \"month\" FROM \"data\"",
        "sales,year,month"
    );
    dql_check(
        query(distinct),
        "SELECT DISTINCT * FROM \"data\""
    );
    dql_check(
        query(all),
        "SELECT * FROM \"data\""
    );
    dql_check(
        query(distinct,all),
        "SELECT * FROM \"data\""
    );
    
    dql_check(
        query(where(1)),
        "SELECT * FROM \"data\" WHERE 1",
        "where(1)"
    );
    dql_check(
        query(where(sales<10)),
        "SELECT * FROM \"data\" WHERE \"sales\"<10",
        "where(sales<10)"
    );
    dql_check(
        query(where((month<=10 or sales>10) and sales>100)),
        R"(SELECT * FROM "data" WHERE (("month"<=10) OR ("sales">10)) AND ("sales">100))",
        "where(((month<=10) or (sales>10)) and (sales>100))"
    );
    dql_check(
        query(where(year+month+10>sales+10)),
        R"(SELECT * FROM "data" WHERE (("year"+"month")+10)>("sales"+10))",
        "where(((year+month)+10)>(sales+10))"
    );
    dql_check(
        query(where(in(month,{"10","11","12"}))),
        R"(SELECT * FROM "data" WHERE month IN (10,11,12)",
        "where(month in [10,11,12])"
    );
    dql_check(
        query(by(year),sum(sales)),
        R"(SELECT "year", sum("sales") FROM "data" GROUP BY "year")",
        "by(year),sum(sales)"
    );
    dql_check(
        query(by(year),by(month),max(sales)),
        R"(SELECT "year", "month", max("sales") FROM "data" GROUP BY "year", "month")",
        "by(year),by(month),max(sales)"
    );
    
    dql_check(
        query(by(year),having(sum(sales)>1000)),
        "SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING sum(\"sales\")>1000",
        "by(year),having(sum(sales)>1000)"
    );
    dql_check(
        query(by(year),having(sum(sales)>1000 and year<2000)),
        "SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING (sum(\"sales\")>1000) AND (\"year\"<2000)",
        "by(year),having((sum(sales)>1000) and (year<2000))"
    );
    dql_check(
        query(by(year),having(sum(sales)>1000),having(year<2000)),
        "SELECT \"year\" FROM \"data\" GROUP BY \"year\" HAVING (sum(\"sales\")>1000) AND (\"year\"<2000)",
        "by(year),having(sum(sales)>1000),having(year<2000)"
    );
    
    dql_check(
        query(order(year),order(sales,-1)),
        "SELECT * FROM \"data\" ORDER BY \"year\" ASC, \"sales\" DESC",
        "order(year),order(sales,-1)"
    );
    dql_check(
        query(by(year),by(month),order(max(sales),-1)),
        "SELECT \"year\", \"month\" FROM \"data\" GROUP BY \"year\", \"month\" ORDER BY max(\"sales\") DESC"
    );

    dql_check(
        query(limit(10)),
        "SELECT * FROM \"data\" LIMIT 10"
    );
    
    dql_check(
        query(offset(10)),
        "SELECT * FROM \"data\" LIMIT 9223372036854775807 OFFSET 10"
    );
    
    dql_check(
        query(by(year),by(month),sum(sales),where(month>6 and year>2000),having(sum(sales)>1000),offset(10),limit(1000)), 
        "SELECT \"year\", \"month\", sum(\"sales\") FROM \"data\" WHERE (\"month\">6) AND (\"year\">2000) GROUP BY \"year\", \"month\" HAVING sum(\"sales\")>1000 LIMIT 1000 OFFSET 10"
    );
}

BOOST_AUTO_TEST_CASE(combiners){
    using namespace DQL;

    Column year = column("year");
    Column month = column("month");
    Column sales = column("sales");

    query(year,sales,limit(10)).execute(data).name("q0");

    query(top(by(year),mean(sales),5)).execute(data).name("q1");

    query(by(year),margin(by(month)),mean(sales)).execute(data).name("q2");

    query(margin(by(year)),margin(by(month)),mean(sales)).execute(data).name("q3");

    query(margin(),by(year),by(month),mean(sales)).execute(data).name("q4");

    query(prop(sum(sales),year),by(month)).execute(data).name("q5");

    data.save("temp.sds");
} 

BOOST_AUTO_TEST_SUITE_END()
