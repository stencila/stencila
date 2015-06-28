#include <boost/test/unit_test.hpp>

#include <stencila/xml.hpp>
using namespace Stencila::Xml;

BOOST_AUTO_TEST_SUITE(xml_patch_quick)

BOOST_AUTO_TEST_CASE(add_basic){
	Document doc(R"(
		<a />
	)");
	doc.patch(R"(
		<add sel="*[1]" type="@href">http://google.com</add>
		<add sel="*[1]">Google</add>
	)");
	BOOST_CHECK_EQUAL(doc.dump(),"<a href=\"http://google.com\">Google</a>");
}

BOOST_AUTO_TEST_CASE(add_append){
	Document doc(R"(
		<div />
	)");
	doc.patch(R"(
		<add sel="*[1]" pos=""><div id="default" /></add>
		<add sel="*[1]" pos="append"><div id="append" /></add>
	)");
	BOOST_CHECK_EQUAL(doc.dump(),R"(<div><div id="default" /><div id="append" /></div>)");
}

BOOST_AUTO_TEST_CASE(add_prepend){
	Document doc(R"(
		<div />
	)");
	doc.patch(R"(
		<add sel="*[1]" pos="prepend"><div id="prepend" /></add>
	)");
	BOOST_CHECK_EQUAL(doc.dump(),R"(<div><div id="prepend" /></div>)");
}

BOOST_AUTO_TEST_CASE(add_before){
	Document doc(R"(
		<div />
	)");
	doc.patch(R"(
		<add sel="*[1]" pos="before">
			<div id="added-1" />
			<div id="added-2" />
			<div id="added-3" />
		</add>
	)");
	BOOST_CHECK_EQUAL(doc.dump(),R"(<div id="added-1" /><div id="added-2" /><div id="added-3" /><div />)");
}

BOOST_AUTO_TEST_CASE(add_after){
	Document doc(R"(
		<div />
	)");
	doc.patch(R"(
		<add sel="*[1]" pos="after">
			<div id="added-1" />
			<div id="added-2" />
			<div id="added-3" />
		</add>
	)");
	BOOST_CHECK_EQUAL(doc.dump(),R"(<div /><div id="added-1" /><div id="added-2" /><div id="added-3" />)");
}

BOOST_AUTO_TEST_CASE(add_nested){
	Document doc(R"(
		<div id="a" />
		<div id="b" >
			<div id="b1">
				<div id="b1a">
					<div id="b1a1">
					</div>
				</div>
			</div>
		</div>
	)");

	doc.patch(R"(
		<add sel="*[1]" pos="append"><a>Hello</a><p>world</p></add>
		<add sel="*[2]//*[1]//*[1]//*[1]" pos="append">foo</add>
	)");
	//" A comment with a quote just to get SublimeText to highlight properly!

	BOOST_CHECK_EQUAL(doc.select("#a a").text(),"Hello");
	BOOST_CHECK_EQUAL(doc.select("#a p").text(),"world");
	BOOST_CHECK_EQUAL(doc.select("#b1a1").text(),"foo");
}

BOOST_AUTO_TEST_CASE(replace){
	Document doc(R"(
		<div />
	)");
	doc.patch(R"(
		<replace sel="*[1]"><div id="replacement" class="foo"/></replace>
		<replace sel="*[1]/@class">bar</replace>
	)");
	BOOST_CHECK_EQUAL(doc.dump(),R"(<div id="replacement" class="bar" />)");
}

BOOST_AUTO_TEST_CASE(remove){
	Document doc(R"(
		<div>
			<div>
			</div>
		</div>
		<a />
	)");
	doc.patch(R"(
		<remove sel="*[1]//*[1]"></remove>
		<remove sel="*[2]"></remove>
	)");
	//" A comment with a quote just to get SublimeText to highlight properly!

	BOOST_CHECK_EQUAL(doc.dump(),R"(<div />)");
}

BOOST_AUTO_TEST_SUITE_END()
