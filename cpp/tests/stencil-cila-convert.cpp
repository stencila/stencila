#include <string>
#include <vector>
#include <iostream>

#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/algorithm/string/iter_find.hpp>

#include <stencila/stencil.hpp>

BOOST_AUTO_TEST_SUITE(stencil_cila_convert_quick)
/**
 * Tests conversion betwen Cila and XML as defined
 * in stencil-cila-convert.txt
 */

using namespace Stencila;


BOOST_AUTO_TEST_CASE(run) {
    // Read the test file
    std::ifstream file("stencil-cila-convert.txt");
    std::stringstream buffer;
	buffer << file.rdbuf();
	std::string text = buffer.str();
    // Split it into individual tests
    std::vector<std::string> tests;
    boost::iter_split(tests, text, boost::first_finder("----------------------------------------------------------------------------------------------------\n"));
    // For each test
    std::ofstream exp("stencil-cila-convert.exp");
    std::ofstream got("stencil-cila-convert.got");
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
    	// Display header section
    	std::string header = sections[0];
    	std::cout<<header;
    	exp<<header<<"--------------------------------------------------\n";
    	got<<header<<"--------------------------------------------------\n";

    	// Determine directionality of test from last two chars of header
    	boost::trim(header);
    	std::string direction = header.substr(header.length()-2);
    	if(not(direction=="<>" or direction==">>" or direction=="<<")) throw std::runtime_error("Invalid directionality: "+direction);

    	// Do tests
    	Stencil stencil;
    	if(direction=="<>" or direction==">>"){
    		stencil.cila(sections[1]);
    		auto html = stencil.html();
    		exp<<sections[2];
    		got<<html;
    		BOOST_CHECK_EQUAL(html,sections[2]);
    	}
    	if(direction=="<>" or direction=="<<"){
    		stencil.html(sections[2]);
    		auto cila = stencil.cila();
    		exp<<sections[1];
    		got<<cila;
    		BOOST_CHECK_EQUAL(cila,sections[1]);
    	}
    	exp<<"----------------------------------------------------------------------------------------------------\n";
    	got<<"----------------------------------------------------------------------------------------------------\n";
    }
}

BOOST_AUTO_TEST_SUITE_END()
