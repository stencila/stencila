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

    Stencil s("file://"+filename);

    BOOST_CHECK_EQUAL(s.title(),"Yo");

    BOOST_CHECK_EQUAL(s.description(),"blah blah blah");

    BOOST_CHECK_EQUAL(s.keywords().size(),3);
    BOOST_CHECK_EQUAL(s.keywords()[1],"b");

    BOOST_CHECK_EQUAL(s.contexts().size(),2);
    BOOST_CHECK_EQUAL(s.contexts()[0],"r");
    BOOST_CHECK_EQUAL(s.contexts()[1],"py");

    BOOST_CHECK_EQUAL(s.authors().size(),2);
    BOOST_CHECK_EQUAL(s.authors()[1],"Slartibartfast");

    BOOST_CHECK_EQUAL(s.theme(),"inter-galatic-journal/theme");

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
#endif

BOOST_AUTO_TEST_CASE(sanitize){
    Stencil s(R"(html://
        <img src="" />
        <div src="" />
        <script></script>
    )");
    s.sanitize();
    BOOST_CHECK(s.one("img[src]"));
    //BOOST_CHECK(not s.one("div[src]"));
    //BOOST_CHECK(not s.one("script"));
}

BOOST_AUTO_TEST_SUITE_END()
