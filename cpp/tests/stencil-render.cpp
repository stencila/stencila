#include <iostream>

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
        context->assign("numbers","1 2 3");
        context->assign("letters","a b c");
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
     * Dump the stencil to std::cerr.
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
        <code class="a" data-exec="map">This should be ignored because no MapContext does not `accept` any code</code>
        <code class="b">This should be ignored because no data-exec attribute</code>
    )");

    BOOST_CHECK(not stencil.select("code.a [data-error]"));
    BOOST_CHECK(not stencil.select("code.b [data-error]"));
}

BOOST_AUTO_TEST_CASE(set){
    render(R"(
        <p data-set="x to 42"></p>
        <p id="x" data-write="x"></p>

        <p data-set="y to 24"></p>
        <p id="y" data-write="y"></p>

        <p id="z" data-set="z"></p>
    )");
    
    BOOST_CHECK_EQUAL(stencil.select("#x").text(),"42");
    BOOST_CHECK_EQUAL(stencil.select("#y").text(),"24");
    BOOST_CHECK_EQUAL(stencil.select("#z [data-error-set-syntax]").text(),"");
}

BOOST_AUTO_TEST_CASE(par){
    render(R"(
        <div data-par="x:number=42" />
        <p id="x" data-write="x"></p>

        <div data-par="y=24" />
        <p id="y" data-write="y"></p>

        <div id="z" data-par="z" />
    )");

    auto input = stencil.select("[data-par=\"x:number=42\"] input");
    BOOST_CHECK_EQUAL(input.attr("name"),"x");
    BOOST_CHECK_EQUAL(input.attr("type"),"number");
    BOOST_CHECK_EQUAL(input.attr("value"),"42");
    BOOST_CHECK_EQUAL(stencil.select("#x").text(),"42");

    BOOST_CHECK_EQUAL(stencil.select("[data-par=\"y=24\"] input[name=\"y\"]").attr("type"),"");
    BOOST_CHECK_EQUAL(stencil.select("#y").text(),"24");

    BOOST_CHECK_EQUAL(stencil.select("[data-par=\"z\"] input[name=\"z\"]").attr("type"),"");
}

BOOST_AUTO_TEST_CASE(text){
    render(R"(
        <p data-write="a" />
        <p data-write="none" />
    )");

    BOOST_CHECK_EQUAL(stencil.select("[data-write=\"a\"]").text(),"A");
    BOOST_CHECK_EQUAL(stencil.select("[data-write=\"none\"]").text(),"");
}

BOOST_AUTO_TEST_CASE(text_lock){
    render(R"(
        <p data-write="a" data-lock="true">So long, and thanks ...</p>
    )");

    BOOST_CHECK_EQUAL(stencil.select("[data-write=\"a\"]").text(),"So long, and thanks ...");
}

/* 

A `data-with` directive can not be tested with map context at present because it does not have a 
`enter(std:string)` method implemented.

BOOST_AUTO_TEST_CASE(with){
    render(R"(
        <ul data-with="planets">
            <li data-write="1" />
            <li data-write="3" />
            <li data-write="5" />
        </ul>
    )");

    BOOST_CHECK_EQUAL(stencil.select("li[data-write=\"1\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(stencil.select("li[data-write=\"3\"]").text(),"Bethselamin");
    BOOST_CHECK_EQUAL(stencil.select("li[data-write=\"5\"]").text(),"Gagrakacka");
}
*/

BOOST_AUTO_TEST_CASE(if_else){
    render(R"(
        <div class="if-off" data-if="none" />
        <div class="else-on" data-elif="a" />
    )");

    BOOST_CHECK(stencil.select("div.if-off").has("data-off"));
    BOOST_CHECK(not stencil.select("div.else-on").has("data-off"));
}

BOOST_AUTO_TEST_CASE(if_elif){
    render(R"(
        <div class="if-on" data-if="a" />
        <div class="elif-off" data-elif="none" />
    )");

    BOOST_CHECK(not stencil.select("div.if-on").has("data-off"));
    BOOST_CHECK(stencil.select("div.elif-off").has("data-off"));
}

BOOST_AUTO_TEST_CASE(if_elif_else){
    render(R"(
        <div class="if-off" data-if="none" />
        <div class="elif-off" data-elif="none" />
        <div class="elif-on" data-elif="a" />
        <div class="else-off" data-else />
    )");

    BOOST_CHECK(stencil.select("div.if-off").has("data-off"));
    BOOST_CHECK(stencil.select("div.elif-off").has("data-off"));
    BOOST_CHECK(not stencil.select("div.elif-on").has("data-off"));
    BOOST_CHECK(stencil.select("div.else-off").has("data-off"));
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

    BOOST_CHECK(stencil.select("p[data-case=\"x\"]").has("data-off"));
    BOOST_CHECK(not stencil.select("p[data-case=\"A\"]").has("data-off"));
    BOOST_CHECK(stencil.select("p[data-case=\"b\"]").has("data-off"));
    BOOST_CHECK(stencil.select("p[data-default]").has("data-off"));
}

BOOST_AUTO_TEST_CASE(switch_2){
    render(R"(
        <div data-switch="a">
            <p data-case="x" />
            <p data-default data-off>
                <span data-write="a" />
            </p>
        </div>
    )");

    BOOST_CHECK(stencil.select("p[data-case=\"x\"]").has("data-off"));
    BOOST_CHECK(not stencil.select("p[data-default]").has("data-off"));
    BOOST_CHECK_EQUAL(stencil.select("p[data-default] span[data-write=\"a\"]").text(),"A");
}

BOOST_AUTO_TEST_CASE(for_){
    render(R"(
        <ul data-for="planet in planets">
            <li data-write="planet" />
        </ul>
    )");
    
    BOOST_CHECK_EQUAL(stencil.select("li[data-index=\"0\"]").text(),"Argabuthon");
    BOOST_CHECK_EQUAL(stencil.select("li[data-index=\"4\"]").text(),"Gagrakacka");
}

BOOST_AUTO_TEST_CASE(for_existing_index){
    render(R"(
        <ul data-for="planet in planets">
            <li data-write="planet" />
            <li data-write="planet" data-index="0">Should be overwritten</li>
        </ul>
    )");
    
    BOOST_CHECK_EQUAL(stencil.select("li[data-index=\"0\"]").text(),"Argabuthon");
}

BOOST_AUTO_TEST_CASE(for_locked_extras){
    render(R"(
        <ul data-for="planet in planets">
            <li data-write="planet" />
            <li data-index="998">Should be removed</li>
            <li data-index="999">Should be retained because contains a lock <span data-lock /> </li>
        </ul>
    )");

    BOOST_CHECK(not stencil.select("li[data-index=\"998\"]"));
    BOOST_CHECK_EQUAL(stencil.select("li[data-index=\"999\"]").attr("data-extra"),"true");
}

BOOST_AUTO_TEST_CASE(for_nested){
    render(R"(
        <tbody data-for="number in numbers">
            <tr data-for="letter in letters">
                <td data-write="letter"></td>
            </tr>
        </tbody
    )");

    BOOST_CHECK_EQUAL(stencil.select("tr[data-index=\"0\"] td[data-index=\"0\"]").text(),"a");
    BOOST_CHECK_EQUAL(stencil.select("tr[data-index=\"1\"] td[data-index=\"1\"]").text(),"b");
    BOOST_CHECK_EQUAL(stencil.select("tr[data-index=\"2\"] td[data-index=\"2\"]").text(),"c");
}

BOOST_AUTO_TEST_CASE(include_simple){
    render(R"(
        <div id="includee">Hello world</div>
        <div data-include="." data-select="#includee" />
    )");
    BOOST_CHECK_EQUAL(stencil.select("[data-include] [data-included] div").text(),"Hello world");
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

    BOOST_CHECK(not stencil.select("[data-include] [data-included] #gone"));
    BOOST_CHECK_EQUAL(stencil.select("[data-include] [data-included] div").text(),"Hello world");
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

    BOOST_CHECK(stencil.select("[data-include] [data-included] #kept"));
    BOOST_CHECK(stencil.select("[data-include] [data-included] #kept-also"));
}

BOOST_AUTO_TEST_CASE(include_simple_rendered){
    render(R"(
        <div id="includee" data-write="a"></div>
        <div data-include="." data-select="#includee" />
    )");

    BOOST_CHECK_EQUAL(stencil.select("[data-include] [data-included] [data-write=\"a\"]").text(),"A");
}

BOOST_AUTO_TEST_CASE(include_modifiers){
    BOOST_TEST_CHECKPOINT("start");

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

    BOOST_CHECK(not stencil.select("div[data-included] #a"));

    BOOST_CHECK(not stencil.select("div[data-included] div#b"));
    BOOST_CHECK(stencil.select("div[data-included] p.b"));

    BOOST_CHECK_EQUAL(stencil.select("div[data-included] div.c strong").text(),"stay the same");

    BOOST_CHECK_EQUAL(stencil.select("div[data-included] div#e").previous().attr("id"),"d");
    BOOST_CHECK_EQUAL(stencil.select("div[data-included] div#e").next().attr("id"),"f");

    BOOST_CHECK_EQUAL(stencil.select("div[data-included] div#g #g1").previous().attr("id"),"g0");
    BOOST_CHECK_EQUAL(stencil.select("div[data-included] div#g #g1").next().attr("id"),"g2");
}

BOOST_AUTO_TEST_CASE(include_par){
    render(R"(
        <div id="includee" data-macro="true">
            <div data-par="x" />
            <div data-par="y=2" />

            <div class="x" data-write="x"></div>
            <div class="y" data-write="y"></div>
            <div class="z" data-write="z"></div>
        </div>

        <div id="a" data-include="." data-select="#includee">
            <p>Required parameter x is missing. Should result in error</p>
        </div>

        <div id="b" data-include="." data-select="#includee">
            <p data-set="x to 10">Parameter value defined in attribute</p>
        </div>

        <div id="c" data-include="." data-select="#includee">
            <p data-set="x to 1" />
            <p data-set="y to 20">Default parameter value overriden</p>
            <p data-set="z to 3">Parameter not declared by stencil author</p>
        </div>
    )");
    
    BOOST_CHECK_EQUAL(stencil.select("#a [data-error-par-required]").attr("data-error-par-required"),"x");
    
    BOOST_CHECK_EQUAL(stencil.select("#b div[data-included] div.x").text(),"10");
    BOOST_CHECK_EQUAL(stencil.select("#b div[data-included] div.y").text(),"2");
    
    BOOST_CHECK_EQUAL(stencil.select("#c div[data-included] div.x").text(),"1");
    BOOST_CHECK_EQUAL(stencil.select("#c div[data-included] div.y").text(),"20");
    BOOST_CHECK_EQUAL(stencil.select("#c div[data-included] div.z").text(),"3");

    // Check that params are removed
    BOOST_CHECK(not stencil.select("#b [data-par]"));
    BOOST_CHECK(not stencil.select("#c [data-par]"));
    BOOST_CHECK(not stencil.select("#d [data-par]"));
}

BOOST_AUTO_TEST_SUITE_END()
