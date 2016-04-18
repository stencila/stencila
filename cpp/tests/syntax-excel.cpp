/**
 * Test for translating Excel (and other) spreadsheet formula
 * to expressions in other languages
 */
#include <boost/test/unit_test.hpp>

#include <stencila/syntax-excel.hpp>
#include <stencila/syntax-r.hpp>
using namespace Stencila::Syntax;

BOOST_AUTO_TEST_SUITE(syntax_excel_quick)

void check(const std::string& in, const std::string& out) {
	ExcelParser p;
	auto n = p.parse(in);
	
	std::ostringstream str;
	ExcelToRGenerator g(str);
	g.visit(n);
	
	BOOST_CHECK_EQUAL(str.str(),out);
}

BOOST_AUTO_TEST_CASE(excel_to_r){
	check("42", "42");
	check("3.14", "3.14");

	check("1+2", "1+2");
	check("1-2", "1-2");
	check("1*2", "1*2");
	check("1/2", "1/2");

	check("A1", "A1");
	check("A1*B1", "A1*B1");

	check("A1:B10", "A1:B10");

	check("SUM(A1:B10)", "sum(A1:B10)");
	check("AVERAGE(A1:B10)", "mean(A1:B10)");
	check("AVERAGE(A1:A10,B1:B10)", "mean(c(A1:A10,B1:B10))");
}

BOOST_AUTO_TEST_SUITE_END()
