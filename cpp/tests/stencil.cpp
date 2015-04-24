#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>

BOOST_AUTO_TEST_SUITE(stencil_quick)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(read){
	std::string filename = (
		boost::filesystem::temp_directory_path()/boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%.html")
	).string();

	std::ofstream out(filename);
	out<<R"(
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
	)";
	out.close();

	BOOST_TEST_CHECKPOINT("start");

	Stencil s("file://"+filename);

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

BOOST_AUTO_TEST_CASE(write_empty){
	Stencil s;
	s.write();
	s.destroy();
}

BOOST_AUTO_TEST_CASE(get){
	Stencil s;
	s.write();
	s.hold(Component::StencilType); // Need to gold this so it is not duplicated on get

	auto instance = s.get(s.address());
	BOOST_CHECK(instance.exists());
	BOOST_CHECK_EQUAL(instance.type(),Component::StencilType);

	Stencil& s_ = instance.as<Stencil>();
	BOOST_CHECK_EQUAL(s.address(),s_.address());

	s.destroy();
}

BOOST_AUTO_TEST_CASE(append){
	Stencil s;

	s.append("span","Don't panic");
	BOOST_CHECK_EQUAL(s.find("span").text(),"Don't panic");

	s.destroy();
}

#if 0
BOOST_AUTO_TEST_CASE(embed){
	Stencil s;
	s.embed();
	
	// Empty element
	div();
	BOOST_CHECK(s.find("div"));

	// Element with text
	span("Don't panic!");
	BOOST_CHECK_EQUAL(s.find("span").text(),"Don't panic!");

	// Element with attributes
	div({{"class","prefect"},{"id","ford"}});
	BOOST_CHECK(s.find("div","class","prefect"));
	BOOST_CHECK(s.select("div#ford.prefect"));

	// Element with attributes and text
	p({{"class","dent"},{"id","arthur"}},"I'm sorry, did you just say you needed my brain?");
	BOOST_CHECK_EQUAL(s.select("p.dent#arthur").text(),"I'm sorry, did you just say you needed my brain?");

	// Nested tags
	div({{"class","advice"}},[](){
		p({{"class","strong"}},"Don't panic!",a({{"href","ddd"}},"Please"),"foo","bar");
		p("Don't panic!","foo","bar");
		p(br(),h1(),h2());
		p([](){
			a();
		});
	});
	BOOST_CHECK(s.select("div.advice"));
	BOOST_CHECK_EQUAL(s.select("div.advice p").text(),"Don't panic!");

	BOOST_CHECK(s.select("div.advice>p>a[href=ddd]"));
	BOOST_CHECK(!s.select("div.advice>a[href=ddd]"));
	
	s.destroy();
}
#endif

BOOST_AUTO_TEST_CASE(sanitize){
	Stencil s(R"(html://
		<img src="" />
		<div src="" />
		<script></script>
	)");
	s.sanitize();
	BOOST_CHECK(s.select("img[src]"));
	//BOOST_CHECK(not s.select("div[src]"));
	//BOOST_CHECK(not s.select("script"));
}

BOOST_AUTO_TEST_SUITE_END()
