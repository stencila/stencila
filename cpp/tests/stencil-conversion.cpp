#include <memory>

#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>
#include <stencila/host.hpp>
#include <stencila/map-context.hpp>
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

BOOST_AUTO_TEST_CASE(compile){
	Stencil s;
	s.cila("Hello world");
	// Prior to rendering (in compile() it is necessary to attach a context)
	s.attach(std::make_shared<MapContext>());
	s.compile();
}

BOOST_AUTO_TEST_SUITE_END()
