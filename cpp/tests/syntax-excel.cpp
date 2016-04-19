/**
 * Test for translating Excel (and other) spreadsheet formula
 * to expressions in other languages
 */
#include <boost/test/unit_test.hpp>

#include <stencila/syntax-excel.hpp>
#include <stencila/syntax-r.hpp>
using namespace Stencila::Syntax;

BOOST_AUTO_TEST_SUITE(syntax_excel_quick)

ExcelParser parser;

BOOST_AUTO_TEST_CASE(excel_to_rsheet){
	ExcelToRSheetGenerator generator;
	#define _(in,out) BOOST_CHECK_EQUAL(generator.generate(parser.parse(in)),out);

	_("42", "42");
	_("3.14", "3.14");

	_("1+2", "1+2");
	_("1-2", "1-2");
	_("1*2", "1*2");
	_("1/2", "1/2");
	_("1^2", "1^2");
	_("1=2", "1==2");
	_("1<>2", "1!=2");

	_("A1", "A1");
	_("$A1", "$A1");
	_("A$1", "A$1");
	_("$A$1", "$A$1");
	_("A1*B1", "A1*B1");

	_("A1:B10", "A1:B10");

	_("SUM(A1:B10)", "SUM(A1:B10)");
	_("AVERAGE(A1:B10)", "AVERAGE(A1:B10)");
	_("AVERAGE(A1:A10,B1:B10)", "AVERAGE(A1:A10,B1:B10)");
}

BOOST_AUTO_TEST_CASE(excel_to_r){
	ExcelToRGenerator generator;
	#define _(in,out) BOOST_CHECK_EQUAL(generator.generate(parser.parse(in)),out);

	_("SUM(A1:B10)", "sum(A1:B10)");
	_("AVERAGE(A1:B10)", "mean(A1:B10)");
	_("AVERAGE(A1:A10,B1:B10)", "mean(c(A1:A10,B1:B10))");
}

BOOST_AUTO_TEST_SUITE_END()
