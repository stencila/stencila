#include <string>
#include <vector>
#include <iostream>

#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/iter_find.hpp>

#include <stencila/stencil.hpp>
#include <stencila/map-context.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_cila_render_quick)
/**
 * Tests rendering of stencils defined using Cila
 * in stencil-cila-render.cila
 */

using namespace Stencila;

BOOST_AUTO_TEST_CASE(run) {
	// Read the test file
	std::ifstream file("stencil-cila-render.cila");
	std::stringstream buffer;
	buffer << file.rdbuf();
	std::string text = buffer.str();
	// Split it into individual tests
	std::vector<std::string> tests;
	boost::iter_split(tests, text, boost::first_finder("--------------------------------------------------\n\n\n"));
	// For each test
	std::ofstream exp("stencil-cila-render.exp");
	std::ofstream got("stencil-cila-render.got");
	for(auto test : tests) {
		/// Split into sections
		std::vector<std::string> sections;
		boost::iter_split(sections, test, boost::first_finder("--------------------------------------------------\n"));
		if(sections.size()!=3){
			std::cerr<<"****************************************\n";
			std::cerr<<test<<std::endl;
			std::cerr<<"****************************************\n";
			throw std::runtime_error("Test does not have 3 sections");
		}
		auto header = sections[0];
		if(header.find("skip")!=std::string::npos) continue;

		// Do tests
		Stencil stencil;
		stencil.cila(sections[1]);
		auto context = std::make_shared<MapContext>();
		stencil.render(context);
		auto rendered = stencil.cila()+"\n";
		auto html = stencil.html()+"\n";

		// Output
		exp<<header<<"--------------------------------------------------\n"
			<<sections[2]<<"--------------------------------------------------\n"
			<<html<<"--------------------------------------------------\n\n\n";
		got<<header<<"--------------------------------------------------\n"
			<<rendered<<"--------------------------------------------------\n"
			<<html<<"--------------------------------------------------\n\n\n";
		if(rendered!=sections[2]){
			std::cerr<<header<<" - failed"<<std::endl;
			BOOST_CHECK_EQUAL(rendered,sections[2]);
		}
	}
}

BOOST_AUTO_TEST_SUITE_END()
