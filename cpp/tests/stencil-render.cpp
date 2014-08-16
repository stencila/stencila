#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
#include <stencila/map-context.hpp>
using namespace Stencila;

/**
 * A fixture for the following rendering tests
 */
struct RenderingFixture {
    Stencil stencil;

    RenderingFixture(void){
        MapContext* context = new MapContext;
        context->assign("a","A");
        context->assign("none","");
        context->assign("planets","Argabuthon Bartledan Bethselamin Earth Gagrakacka");
        stencil.attach(context);
    }

    ~RenderingFixture(void){
        stencil.destroy();
    }

    /**
     * Render the stencil in the map context
     */
    void render(const std::string html){
        stencil.html(html);
        stencil.render();
    }

    /**
     * Dump the stecnil to std::cerr.
     * Useful to put in a test to work out why a test has failed.
     */
    void dump(void){
        std::cerr
            <<"-----------------------------------\n"
            <<stencil.html()
            <<"-----------------------------------\n"
            <<std::flush;
    }

};

BOOST_FIXTURE_TEST_SUITE(stencil_render,RenderingFixture)

using namespace Stencila;

BOOST_AUTO_TEST_CASE(code){
    render(R"(
        <code class="a" data-code="map">This should be ignored because no MapContext does not `accept` any code</code>
        <code class="b">This should be ignored because no data-code attribute</code>
    )");

    BOOST_CHECK(not stencil.one("code.a [data-error]"));
    BOOST_CHECK(not stencil.one("code.b [data-error]"));
}

BOOST_AUTO_TEST_CASE(text){
    render(R"(
        <p data-text="a" />
        <p data-text="none" />
    )");

    BOOST_CHECK_EQUAL(stencil.one("[data-text=\"a\"]").text(),"A");
    BOOST_CHECK_EQUAL(stencil.one("[data-text=\"none\"]").text(),"");
}

BOOST_AUTO_TEST_CASE(text_lock){
    render(R"(
        <p data-text="a" data-lock="true">So long, and thanks ...</p>
    )");

    BOOST_CHECK_EQUAL(stencil.one("[data-text=\"a\"]").text(),"So long, and thanks ...");
}

/* 

A `data-with` directive can not be tested with map context at present because it does not have a 
`enter(std:string)` method implemented.

BOOST_AUTO_TEST_CASE(with){
    render(R"(
        <ul data-with="planets">
            <li data-text="1" />
            <li data-text="3" />
            <li data-text="5" />
        </ul>
    )");

    BOOST_CHECK_EQUAL(stencil.one("li[data-text=\"1\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(stencil.one("li[data-text=\"3\"]").text(),"Bethselamin");
    BOOST_CHECK_EQUAL(stencil.one("li[data-text=\"5\"]").text(),"Gagrakacka");
}
*/

BOOST_AUTO_TEST_CASE(if_else){
    render(R"(
        <div class="if-off" data-if="none" />
        <div class="else-on" data-elif="a" />
    )");

    BOOST_CHECK(stencil.one("div.if-off").has("data-off"));
    BOOST_CHECK(not stencil.one("div.else-on").has("data-off"));
}

BOOST_AUTO_TEST_CASE(if_elif){
    render(R"(
        <div class="if-on" data-if="a" />
        <div class="elif-off" data-elif="none" />
    )");

    BOOST_CHECK(not stencil.one("div.if-on").has("data-off"));
    BOOST_CHECK(stencil.one("div.elif-off").has("data-off"));
}

BOOST_AUTO_TEST_CASE(if_elif_else){
    render(R"(
        <div class="if-off" data-if="none" />
        <div class="elif-off" data-elif="none" />
        <div class="elif-on" data-elif="a" />
        <div class="else-off" data-else />
    )");

    BOOST_CHECK(stencil.one("div.if-off").has("data-off"));
    BOOST_CHECK(stencil.one("div.elif-off").has("data-off"));
    BOOST_CHECK(not stencil.one("div.elif-on").has("data-off"));
    BOOST_CHECK(stencil.one("div.else-off").has("data-off"));
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

    BOOST_CHECK(stencil.one("p[data-case=\"x\"]").has("data-off"));
    BOOST_CHECK(not stencil.one("p[data-case=\"A\"]").has("data-off"));
    BOOST_CHECK(stencil.one("p[data-case=\"b\"]").has("data-off"));
    BOOST_CHECK(stencil.one("p[data-default]").has("data-off"));
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

    BOOST_CHECK(stencil.one("p[data-case=\"x\"]").has("data-off"));
    BOOST_CHECK(not stencil.one("p[data-default]").has("data-off"));
    BOOST_CHECK_EQUAL(stencil.one("p[data-default] span[data-text=\"a\"]").text(),"A");
}

BOOST_AUTO_TEST_CASE(for_){
    render(R"(
        <ul data-for="planet:planets">
            <li data-text="planet" />
        </ul>
    )");
    
    BOOST_CHECK_EQUAL(stencil.one("li[data-index=\"0\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(stencil.one("li[data-index=\"4\"]").text(),"Gagrakacka");
}

BOOST_AUTO_TEST_CASE(for_existing_index){
    render(R"(
        <ul data-for="planet:planets">
            <li data-text="planet" />
            <li data-text="planet" data-index="0">Should be overwritten</li>
        </ul>
    )");
    
    BOOST_CHECK_EQUAL(stencil.one("li[data-index=\"0\"]").text(),"Argabuthon");
}

BOOST_AUTO_TEST_CASE(for_locked_extras){
    render(R"(
        <ul data-for="planet:planets">
            <li data-text="planet" />
            <li data-index="998">Should be removed</li>
            <li data-index="999">Should be retained because contains a lock <span data-lock /> </li>
        </ul>
    )");

    BOOST_CHECK(not stencil.one("li[data-index=\"998\"]"));
    BOOST_CHECK_EQUAL(stencil.one("li[data-index=\"999\"]").attr("data-extra"),"true");
}

BOOST_AUTO_TEST_CASE(include_simple){
    render(R"(
        <div id="includee">Hello world</div>
        <div data-include="." data-select="#includee" />
    )");

    BOOST_CHECK_EQUAL(stencil.one("[data-include] [data-included] div").text(),"Hello world");
}

BOOST_AUTO_TEST_CASE(include_previous_included_is_cleared){
    render(R"(
        <div id="includee">Hello world</div>
        <div data-include="." data-select="#includee">
            <div data-included>
                <span id="gone">This should be removed</span>
            </div>
        </div>
    )");

    BOOST_CHECK(not stencil.one("[data-include] [data-included] #gone"));
    BOOST_CHECK_EQUAL(stencil.one("[data-include] [data-included] div").text(),"Hello world");
}

BOOST_AUTO_TEST_CASE(include_previous_included_is_not_cleared_if_lock){
    render(R"(
        <div id="includee">Hello world</div>
        <div data-include="." data-select="#includee">
            <div data-included>
                <span id="kept" data-lock="true">This should NOT be removed because it has a data-lock</span>
                <span id="kept-also"></span>
            </div>
        </div>
    )");

    BOOST_CHECK(stencil.one("[data-include] [data-included] #kept"));
    BOOST_CHECK(stencil.one("[data-include] [data-included] #kept-also"));
}

BOOST_AUTO_TEST_CASE(include_simple_rendered){
    render(R"(
        <div id="includee" data-text="a"></div>
        <div data-include="." data-select="#includee" />
    )");

    BOOST_CHECK_EQUAL(stencil.one("[data-include] [data-included] [data-text=\"a\"]").text(),"A");
}

BOOST_AUTO_TEST_CASE(include_modifiers){
    render(R"(
        <div id="includee">
            <div id="a" />
            <div id="b" />
            <div id="c" class="c" />
            <div id="e" />
            <div id="g">
                <div id="g1" />
            </div>
        </div>

        <div data-include="." data-select="#includee">
            <div data-delete="#a" />
            <p data-replace="#b" class="b">
                This should replace div#b with p.b
            </p>
            <div data-change="#c">
                This should replace the contents of div#c but its attributes
                should <strong>stay the same</strong>.
            </div>
            <div data-before="#e" id="d" />
            <div data-after="#e" id="f" />
            <div data-prepend="#g" id="g0" />
            <div data-append="#g" id="g2" />
        </div>
    )");

    BOOST_CHECK(not stencil.one("div[data-included] #a"));

    BOOST_CHECK(not stencil.one("div[data-included] div#b"));
    BOOST_CHECK(stencil.one("div[data-included] p.b"));

    BOOST_CHECK_EQUAL(stencil.one("div[data-included] div.c strong").text(),"stay the same");

    BOOST_CHECK_EQUAL(stencil.one("div[data-included] div#e").previous().attr("id"),"d");
    BOOST_CHECK_EQUAL(stencil.one("div[data-included] div#e").next().attr("id"),"f");

    BOOST_CHECK_EQUAL(stencil.one("div[data-included] div#g #g1").previous().attr("id"),"g0");
    BOOST_CHECK_EQUAL(stencil.one("div[data-included] div#g #g1").next().attr("id"),"g2");
}

BOOST_AUTO_TEST_CASE(include_arg){
    render(R"(
        <div id="includee" data-macro="true">
            <div data-arg="x" />
            <div data-arg="y:2" />

            <div class="x" data-text="x"></div>
            <div class="y" data-text="y"></div>
            <div class="z" data-text="z"></div>
        </div>

        <div id="a" data-include="." data-select="#includee">
            <p>Required argument x is missing. Should result in error</p>
        </div>

        <div id="b" data-include="." data-select="#includee">
            <p data-set="x:10">Argument value defined in attribute</p>
        </div>

        <div id="c" data-include="." data-select="#includee">
            <p data-set="x">11 (Argument value defined in text)</p>
        </div>

        <div id="d" data-include="." data-select="#includee">
            <p data-set="x:1" />
            <p data-set="y:20">Default parameter value overriden</p>
            <p data-set="z:3">Argument not declared by stencil author</p>
        </div>
    )");
    //dump();
    BOOST_CHECK_EQUAL(stencil.one("#a div[data-included] div[data-error=\"arg-required\"]").attr("data-arg"),"x");
    
    BOOST_CHECK_EQUAL(stencil.one("#b div[data-included] div.x").text(),"10");
    BOOST_CHECK_EQUAL(stencil.one("#b div[data-included] div.y").text(),"2");

    BOOST_CHECK_EQUAL(stencil.one("#c div[data-included] div.x").text(),"11 (Argument value defined in text)");
    BOOST_CHECK_EQUAL(stencil.one("#c div[data-included] div.y").text(),"2");
    
    BOOST_CHECK_EQUAL(stencil.one("#d div[data-included] div.x").text(),"1");
    BOOST_CHECK_EQUAL(stencil.one("#d div[data-included] div.y").text(),"20");
    BOOST_CHECK_EQUAL(stencil.one("#d div[data-included] div.z").text(),"3");

    // Check that params are removed
    BOOST_CHECK(not stencil.one("#b [data-arg]"));
    BOOST_CHECK(not stencil.one("#c [data-arg]"));
    BOOST_CHECK(not stencil.one("#d [data-arg]"));
}

BOOST_AUTO_TEST_SUITE_END()
