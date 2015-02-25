#include <iostream>

#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_attrs_quick)

BOOST_AUTO_TEST_CASE(general){
	Stencil s(R"(html://
	<html>
		<body>
			<main id="content">
				<div id="title">Yo</div>
				<div id="description">blah blah blah</div>
				<div id="keywords">a,b,cd<div>
				<div class="author">Arthur Dent</div>
				<div class="author">Slartibartfast</div>
				<div id="contexts">r,py</div>
				<div id="theme">inter-galatic-journal/theme</div>
				<p class="advice">Don't panic!</p>
			</main>
		</body>
	</html>
	)");

	BOOST_CHECK_EQUAL(s.title(),"Yo");

	BOOST_CHECK_EQUAL(s.description(),"blah blah blah");

	BOOST_CHECK_EQUAL(s.keywords().size(),3u);
	BOOST_CHECK_EQUAL(s.keywords()[1],"b");

	BOOST_CHECK_EQUAL(s.contexts().size(),2u);
	BOOST_CHECK_EQUAL(s.contexts()[0],"r");
	BOOST_CHECK_EQUAL(s.contexts()[1],"py");

	BOOST_CHECK_EQUAL(s.authors().size(),2u);
	BOOST_CHECK_EQUAL(s.authors()[1],"Slartibartfast");

	BOOST_CHECK_EQUAL(s.theme(),"inter-galatic-journal/theme");

	BOOST_CHECK_EQUAL(s.select("p.advice").text(),"Don't panic!");

	s.destroy();
}

BOOST_AUTO_TEST_CASE(contexts){
	{
		Stencil s;
		BOOST_CHECK_EQUAL(s.contexts().size(),0u);
	}{
		Stencil s(R"(html://
			<div id="contexts">r,py</div>
			<pre data-exec="foo"></pre>
		)");

		BOOST_CHECK_EQUAL(s.contexts().size(),2u);
		BOOST_CHECK_EQUAL(s.contexts()[0],"r");
		BOOST_CHECK_EQUAL(s.contexts()[1],"py");
	}{
		Stencil s(R"(html://
			<pre data-exec="r"></pre>
		)");
		BOOST_CHECK_EQUAL(s.contexts().size(),1u);
		BOOST_CHECK_EQUAL(s.contexts()[0],"r");
	}{
		Stencil s(R"(html://
			<pre data-exec="r,py"></pre>
			<pre data-exec="py"></pre>
			<pre data-exec="py,r"></pre>
		)");
		BOOST_CHECK_EQUAL(s.contexts().size(),2u);
		BOOST_CHECK_EQUAL(s.contexts()[0],"py");
		BOOST_CHECK_EQUAL(s.contexts()[1],"r");
	}{
		Stencil s(R"(html://
			<pre data-exec="r"></pre>
			<pre data-exec="py"></pre>
			<pre data-exec="py"></pre> 
		)");
		BOOST_CHECK_EQUAL(s.contexts().size(),2u);
		BOOST_CHECK_EQUAL(s.contexts()[0],"py");
		BOOST_CHECK_EQUAL(s.contexts()[1],"r");
	}
}

BOOST_AUTO_TEST_SUITE_END()
