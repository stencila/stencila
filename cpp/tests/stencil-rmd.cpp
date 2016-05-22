#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_rmd)

BOOST_AUTO_TEST_CASE(to){
	Stencil s;

	for(auto pair : std::vector<std::array<std::string,2>>{
		{"``` {r}\nx = 42\n```\n", "<pre data-exec=\"r\" data-rmd=\"{r}\">x = 42\n</pre>"},
		{"``` {r label, eval=FALSE}\n```\n", "<pre data-exec=\"r off\" data-rmd=\"{r label, eval=FALSE}\"></pre>"},
		{"``` {r eval=T, echo=T}\n```\n", "<pre data-exec=\"r show\" data-rmd=\"{r eval=T, echo=T}\"></pre>"},
		{"``` {r fig.width=10}\n```\n", "<pre data-exec=\"r width 10in\" data-rmd=\"{r fig.width=10}\"></pre>"},

		{"`r x`\n", "<p><span data-text=\"x\"></span></p>"}
	}) {
		s.rmd(pair[0]);
		BOOST_CHECK_EQUAL(s.html(), pair[1]);
		BOOST_CHECK_EQUAL(s.rmd(), pair[0]);
	}
}

BOOST_AUTO_TEST_SUITE_END()
