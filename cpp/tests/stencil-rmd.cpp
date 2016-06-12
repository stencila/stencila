#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_rmd)

// Bidirectional transaltion tests
BOOST_AUTO_TEST_CASE(from_to){
	Stencil s;

	std::vector<std::array<std::string,2>> pairs = {
		// Code chuncks
		{"``` {r}\nx = 42\n```\n", "<pre data-exec=\"r\">x = 42\n</pre>"},
		{"``` {r eval=FALSE}\n```\n", "<pre data-exec=\"r off\"></pre>"},
		{"``` {r echo=TRUE}\n```\n", "<pre data-exec=\"r show\"></pre>"},
		{"``` {r dev=\"png\"}\n```\n", "<pre data-exec=\"r format png\"></pre>"},
		{"``` {r fig.width=10}\n```\n", "<pre data-exec=\"r width 10in\"></pre>"},
		{"``` {r fig.width=10, fig.height=10}\n```\n", "<pre data-exec=\"r width 10in height 10in\"></pre>"},
		{"``` {r fig.width=10, unsupported.option=2}\n```\n", "<pre data-exec=\"r width 10in\" data-extra=\"unsupported.option=2\"></pre>"},
		// Inline code
		{"`r x`\n", "<p><span data-text=\"x\"></span></p>"}
	};

	for(auto pair : pairs) {
		s.rmd(pair[0]);
		BOOST_CHECK_EQUAL(s.html(), pair[1]);
		BOOST_CHECK_EQUAL(s.rmd(), pair[0]);
	}
}

// Unidrectional test to check for parsing and handling of 
// RMarkdown options with default values
BOOST_AUTO_TEST_CASE(from){
	Stencil s;

	std::vector<std::array<std::string,2>> pairs = {
		{"``` {r eval=TRUE}\nx = 42\n```\n", "<pre data-exec=\"r\">x = 42\n</pre>"},
		{"``` {r eval=TRUE, echo=TRUE}\n```\n", "<pre data-exec=\"r show\"></pre>"},
		{"``` {r echo=FALSE, fig.width=10, eval=TRUE}\n```\n", "<pre data-exec=\"r width 10in\"></pre>"},
	};

	for(auto pair : pairs) {
		s.rmd(pair[0]);
		BOOST_CHECK_EQUAL(s.html(), pair[1]);
	}
}

BOOST_AUTO_TEST_SUITE_END()
