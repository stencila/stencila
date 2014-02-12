#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>

BOOST_AUTO_TEST_SUITE(stencil)

using namespace Stencila;


BOOST_AUTO_TEST_CASE(read){
    std::string filename = (
        boost::filesystem::temp_directory_path()/boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%.html")
    ).string();

    std::ofstream out(filename);
    out<<R"(
    <html>
        <head>
            <title>Yo</title>
            <meta name="keywords" content="a,b,cd" />
            <meta name="description" content="blah blah blah" />
        </head>
        <body>
            <ul id="contexts">
                <li>r</li>
                <li>py</li>
            </ul>
            <address id="authors">
                <a rel="author">Arthur Dent</a>
                <a rel="author">Slartibartfast</a>
            </address>
            <main id="content">
                <p class="advice">Don't panic!</p>
            </main>
        </body>
    </html>
    )";
    out.close();

    Stencil s;
    s.read(filename);

    BOOST_CHECK_EQUAL(s.title(),"Yo");

    BOOST_CHECK_EQUAL(s.keywords().size(),3);
    BOOST_CHECK_EQUAL(s.keywords()[1],"b");

    BOOST_CHECK_EQUAL(s.description(),"blah blah blah");

    BOOST_CHECK_EQUAL(s.contexts().size(),2);
    BOOST_CHECK_EQUAL(s.contexts()[0],"r");
    BOOST_CHECK_EQUAL(s.contexts()[1],"py");

    BOOST_CHECK_EQUAL(s.authors().size(),2);

    BOOST_CHECK_EQUAL(s.one("p.advice").text(),"Don't panic!");

    s.destroy();
}

BOOST_AUTO_TEST_CASE(write_empty){
    Stencil s;
    s.write();
    s.destroy();
}

BOOST_AUTO_TEST_CASE(append){
    Stencil s;

	s.append("span","Don't panic");
    BOOST_CHECK_EQUAL(s.find("span").text(),"Don't panic");

    s.destroy();
}

BOOST_AUTO_TEST_CASE(embed){
    Stencil s;
    s.embed();
    using namespace Embed;

    // Empty element
    div();
    BOOST_CHECK(s.find("div"));

    // Element with text
    span("Don't panic!");
    BOOST_CHECK_EQUAL(s.find("span").text(),"Don't panic!");

    // Element with attributes
    div({{"class","prefect"},{"id","ford"}});
    BOOST_CHECK(s.find("div","class","prefect"));
    BOOST_CHECK(s.one("div#ford.prefect"));

    // Element with attributes and text
    p({{"class","dent"},{"id","arthur"}},"I'm sorry, did you just say you needed my brain?");
    BOOST_CHECK_EQUAL(s.one("p.dent#arthur").text(),"I'm sorry, did you just say you needed my brain?");

    // Nested tags
    div({{"class","advice"}},[](){
    	p({{"class","strong"}},"Don't panic!",a({{"href","ddd"}},"Please"),"foo","bar");
        p("Don't panic!","foo","bar");
        p(br(),h1(),h2());
        p([](){
            a();
        });
   	});
    BOOST_CHECK(s.one("div.advice"));
    BOOST_CHECK_EQUAL(s.one("div.advice p").text(),"Don't panic!");

    BOOST_CHECK(s.one("div.advice>p>a[href=ddd]"));
    BOOST_CHECK(!s.one("div.advice>a[href=ddd]"));
    
    s.destroy();
}

BOOST_AUTO_TEST_CASE(sanitize){
    Stencil s;

    s.sanitize();

    s.destroy();
}

BOOST_AUTO_TEST_SUITE_END()
 