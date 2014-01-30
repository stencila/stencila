#include <iostream>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/utilities/xml.hpp>

BOOST_AUTO_TEST_SUITE(formats_xml)

using namespace Stencila::Utilities::Xml;

BOOST_AUTO_TEST_CASE(attributes){
    Document doc;
    Node div = doc.append("div");

    BOOST_CHECK_EQUAL(div.attr("class"),"");

    div.attr("class","foo");
    BOOST_CHECK_EQUAL(div.attr("class"),"foo");

    div.add("class","bar");
    BOOST_CHECK_EQUAL(div.attr("class"),"foo bar");

    div.erase("class");
    BOOST_CHECK_EQUAL(div.attr("class"),"");
}

BOOST_AUTO_TEST_CASE(append){
    Document doc;

    //Just an element
    doc.append("div");
    BOOST_CHECK(doc.find("div"));

    //An element with some text
    doc.append("div","hello");
    
    //An element with some attributes
    doc.append("div",{{"class","ba"},{"data-foo","false"}});
    BOOST_CHECK(doc.find("div","class","ba"));
    BOOST_CHECK(doc.find("div","data-foo","false"));
    
    //An element with some attributes and some text
    doc.append("div",{{"class","bb"}},"hello");
    BOOST_CHECK(doc.find("div","class","bb"));
}

BOOST_AUTO_TEST_CASE(remove){

}

BOOST_AUTO_TEST_CASE(clear){

}

/**
 * @class Node
 * 
 * Test the translation of CSS selectors to XPath
 * These tests are based on those in Python's [cssselect](https://pypi.python.org/pypi/cssselect) package
 * See the [test_translation function](https://github.com/SimonSapin/cssselect/blob/master/cssselect/tests.py#L314)
 */
BOOST_AUTO_TEST_CASE(xpath){
    #define CHECK(selector,xpat) BOOST_CHECK_EQUAL(Node::xpath(selector),"descendant-or-self::" xpat);

    CHECK("*",                  "*")
    CHECK("e",                  "e")

    CHECK("e[foo]",             "e[@foo]")

    CHECK("e[foo=bar]",         "e[@foo='bar']")
    CHECK("e[foo='foo bar']",   "e[@foo='foo bar']")
    CHECK("e[foo=\"foo bar\"]", "e[@foo='foo bar']")

    CHECK("e[foo~='bar']",      "e[@foo and contains(concat(' ',normalize-space(@foo),' '),' bar ')]")
    CHECK("e[foo^='bar']",      "e[@foo and starts-with(@foo,'bar')]")
    CHECK("e[foo$='bar']",      "e[@foo and substring(@foo,string-length(@foo)-2)='bar']")
    CHECK("e[foo*='bar']",      "e[@foo and contains(@foo,'bar')]")
    CHECK("e[foo|='bar']",      "e[@foo and (@foo='bar' or starts-with(@foo,'bar-'))]")

    CHECK("e.myclass",          "e[@class and contains(concat(' ',normalize-space(@class),' '),' myclass ')]")
    CHECK("e.my-class",          "e[@class and contains(concat(' ',normalize-space(@class),' '),' my-class ')]")
    CHECK("e#myid",             "e[@id='myid']")
    CHECK("e#my-id",             "e[@id='my-id']")

    CHECK("e f",                "e/descendant::f")
    CHECK("e > f",              "e/f")
    CHECK("e + f",              "e/following-sibling::*[name()='f' and (position()=1)]")
    CHECK("e ~ f",              "e/following-sibling::f")
    CHECK("div#container p",    "div[@id='container']/descendant::p")
    
    #undef CHECK
}

BOOST_AUTO_TEST_CASE(one){
    Document doc;
    doc.load(R"(
        <html>
            <div class="a">A</div>
            <div class="a">This is the second div.a so should not be selected</div>

            <span id="b">B</span>

            <div id="c">
                <div foo="bar">C</div>                
                <div foo="bar foo">D</div>
            </div>
        </html>
    )");

    #define CHECK(selector,result) BOOST_CHECK_EQUAL(doc.one(selector).text(),result);

    CHECK("div.a","A")
    
    CHECK("#b","B")
    CHECK("span#b","B")
    CHECK("div.a + span","B")
    
    CHECK("div#c div[foo]","C")
    CHECK("div#c div[foo=bar]","C")
    CHECK("div#c>div","C")

    CHECK("div[foo='bar foo']","D")
    
    #undef CHECK
}

BOOST_AUTO_TEST_CASE(dump){
    Document doc;
    std::string content = "<div class=\"foo\">Hello world</div>";
    doc.load(content);
    BOOST_CHECK_EQUAL(doc.dump(),content);
}

BOOST_AUTO_TEST_SUITE_END()
