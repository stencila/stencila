#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
#include <stencila/contexts/map.hpp>

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

/**
 * A fixture for the follwoing rendering tests
 */
struct RenderingFixture {
    Stencila::Contexts::Map map;
    Stencila::Stencil s;

    RenderingFixture(void){
        map.assign("a","A");
        map.assign("none","");

        map.assign("planets","");
        map.enter("planets");
            map.assign("1","Argabuthon");
            map.assign("2","Bartledan");
            map.assign("3","Bethselamin");
            map.assign("4","Earth");
            map.assign("5","Gagrakacka");
        map.exit();
    }

    ~RenderingFixture(void){
        s.destroy();
    }

    /**
     * Render the stencil in the map context
     */
    void render(const std::string html){
        s.append_html(html);
        s.render(map);
    }

    /**
     * Dump the sentecnil to std::cerr.
     * Useful to put in a test to work out why a test has failed.
     */
    void dump(void){
        std::cerr
            <<"-----------------------------------\n"
            <<s.dump(true)
            <<"-----------------------------------\n"
            <<std::flush;
    }

};

BOOST_FIXTURE_TEST_SUITE(rendering,RenderingFixture)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(code){
    render(R"(
        <code class="executed" data-code="map" />
        <code class="ignored" />
    )");

    BOOST_CHECK_EQUAL(s.one("code.executed").attr("data-error"),"Not supported by context type: map-context");
    BOOST_CHECK_EQUAL(s.one("code.ignored").attr("data-error"),"");
}

BOOST_AUTO_TEST_CASE(text){
    render(R"(
        <p data-text="a" />
        <p data-text="none" />
    )");

    BOOST_CHECK_EQUAL(s.one("[data-text=\"a\"]").text(),"A");
    BOOST_CHECK_EQUAL(s.one("[data-text=\"none\"]").text(),"");
}

BOOST_AUTO_TEST_CASE(text_lock){
    render(R"(
        <p data-text="a" data-lock="true">So long, and thanks ...</p>
    )");

    BOOST_CHECK_EQUAL(s.one("[data-text=\"a\"]").text(),"So long, and thanks ...");
}

BOOST_AUTO_TEST_CASE(image){
    render(R"(
        <code data-image="svg">
            plot(x,y)
        </code>
    )");

    BOOST_CHECK_EQUAL(s.one("code").attr("data-error"),"Not supported by context type: map-context");
}

BOOST_AUTO_TEST_CASE(with){
    render(R"(
        <ul data-with="planets">
            <li data-text="1" />
            <li data-text="3" />
            <li data-text="5" />
        </ul>
    )");

    BOOST_CHECK_EQUAL(s.one("li[data-text=\"1\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(s.one("li[data-text=\"3\"]").text(),"Bethselamin");
    BOOST_CHECK_EQUAL(s.one("li[data-text=\"5\"]").text(),"Gagrakacka");
}

BOOST_AUTO_TEST_CASE(if_else){
    render(R"(
        <div class="if-off" data-if="none" />
        <div class="else-on" data-elif="a" />
    )");

    BOOST_CHECK(s.one("div.if-off").has("data-off"));
    BOOST_CHECK(not s.one("div.else-on").has("data-off"));
}

BOOST_AUTO_TEST_CASE(if_elif){
    render(R"(
        <div class="if-on" data-if="a" />
        <div class="elif-off" data-elif="none" />
    )");

    BOOST_CHECK(not s.one("div.if-on").has("data-off"));
    BOOST_CHECK(s.one("div.elif-off").has("data-off"));
}

BOOST_AUTO_TEST_CASE(if_elif_else){
    render(R"(
        <div class="if-off" data-if="none" />
        <div class="elif-off" data-elif="none" />
        <div class="elif-on" data-elif="a" />
        <div class="else-off" data-else />
    )");

    BOOST_CHECK(s.one("div.if-off").has("data-off"));
    BOOST_CHECK(s.one("div.elif-off").has("data-off"));
    BOOST_CHECK(not s.one("div.elif-on").has("data-off"));
    BOOST_CHECK(s.one("div.else-off").has("data-off"));
}

BOOST_AUTO_TEST_CASE(switch_1){
    render(R"(
        <div data-switch="a">
            <p data-case="x" />
            <p data-case="A" />
            <p data-case="b" />
            <p data-default />
        </div>
    )");

    BOOST_CHECK(s.one("p[data-case=\"x\"]").has("data-off"));
    BOOST_CHECK(not s.one("p[data-case=\"A\"]").has("data-off"));
    BOOST_CHECK(s.one("p[data-case=\"b\"]").has("data-off"));
    BOOST_CHECK(s.one("p[data-default]").has("data-off"));
}

BOOST_AUTO_TEST_CASE(switch_2){
    render(R"(
        <div data-switch="a">
            <p data-case="x" />
            <p data-default data-off>
                <span data-text="a" />
            </p>
        </div>
    )");

    BOOST_CHECK(s.one("p[data-case=\"x\"]").has("data-off"));
    BOOST_CHECK(not s.one("p[data-default]").has("data-off"));
    BOOST_CHECK_EQUAL(s.one("p[data-default] span[data-text=\"a\"]").text(),"A");
}

BOOST_AUTO_TEST_CASE(for_){
    render(R"(
        <ul data-for="planet:planets">
            <li data-each data-text="planet" />
        </ul>
    )");
    
    BOOST_CHECK_EQUAL(s.one("li[data-index=\"0\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(s.one("li[data-index=\"4\"]").text(),"Gagrakacka");
}

BOOST_AUTO_TEST_CASE(for_existing_index){
    render(R"(
        <ul data-for="planet:planets">
            <li data-each data-text="planet" />
            <li data-text="planet" data-index="0"><span>(the first)</span></li>
        </ul>
    )");
    
    BOOST_CHECK_EQUAL(s.one("li[data-index=\"0\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(s.one("li[data-index=\"0\"] span").text(),"(the first)");
    BOOST_CHECK(not s.one("li[data-index=\"1\"] span"));
}

BOOST_AUTO_TEST_CASE(for_locked_extras){
    render(R"(
        <ul data-for="planet:planets">
            <li data-each data-text="planet" />
            <li data-index="998">Should be removed</li>
            <li data-index="999">Should be retained because contains a lock <span data-lock /> </li>
        </ul>
    )");

    BOOST_CHECK(not s.one("li[data-index=\"998\"]"));
    BOOST_CHECK_EQUAL(s.one("li[data-index=\"999\"]").attr("data-extra"),"true");
}

/**
 * @todo a bunch more include related tests
 */
BOOST_AUTO_TEST_CASE(include){

    Stencil s1(R"(html://
        <div class="inner">
            <div class="a" data-text="a"></div>
            <div class="b">B</div>
            <div class="c"></div>
            <div class="x" data-text="x"></div>
            <div class="y" data-text="y"></div>
        </div>
    )");

    s1.path("s1");

    render(R"(
        <div data-text="a"></div>
        <div data-include="s1" data-select=".inner *">
            <p data-delete=".b" />
            <p data-replace=".c">CCCCCCCC</p>
            <p data-param="x:5" />
            <p data-param="y">Oh, why!</p>
        </div>
    )");

    //dump();
    BOOST_CHECK(s.one("div[data-include] div[data-included]"));
}

BOOST_AUTO_TEST_SUITE_END()

