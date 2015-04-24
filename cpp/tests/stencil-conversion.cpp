#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>
#include <stencila/host.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_conversion_slow)

BOOST_AUTO_TEST_CASE(to){
	Stencil s;
	s.cila("Hello world, here is an equation |e = mc^2| in line");

	auto docx = Host::temp_filename("docx");
	s.docx("to",docx);

	auto pdf = Host::temp_filename("pdf");
	s.pdf("to",pdf);

	auto tn = Host::temp_filename("png");
	s.preview(tn);
}

BOOST_AUTO_TEST_CASE(from_markdown){
	Stencil s;
	
	auto md = Host::temp_filename("md");
	std::ofstream f(md);
	f<<R"(
# Heading 1

`print()`
	)";
	f.close();
	s.markdown("from",md);

	BOOST_CHECK_EQUAL(s.html(),"");
}

BOOST_AUTO_TEST_SUITE_END()
