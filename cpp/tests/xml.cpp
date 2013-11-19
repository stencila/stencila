#include <iostream>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/xml.hpp>

struct formatsXmlFixture {

    Stencila::Xml::Document doc;
    
    formatsXmlFixture(void){
        doc.load(
            "<div class='a'>"
            "   <div class='aa'/>"
            "   <div class='ab' data-print='x'>text</div>"
            "</div>"
        );
    }    
};

BOOST_FIXTURE_TEST_SUITE(formats_xml,formatsXmlFixture)

using namespace Stencila::Xml;

BOOST_AUTO_TEST_CASE(select_next_sibling){
    //Selects node using next_sibling
    auto node = doc.child("div").child("div").next_sibling();
    BOOST_CHECK_EQUAL(node.child_value(),"text");
    BOOST_CHECK_EQUAL(node.attribute("data-print").value(),"x");
}

BOOST_AUTO_TEST_CASE(select_xpath){
    //Selects node using Xpath
    auto node = doc.select_single_node("//div[@class='ab']").node();
    BOOST_CHECK_EQUAL(node.child_value(),"text");
    BOOST_CHECK_EQUAL(node.attribute("data-print").value(),"x");
}

BOOST_AUTO_TEST_CASE(select_css_translate){
    //Translate CSS selector to XPath selector
    
    /*!
    @todo Generate a large number of test cases using a Python script with cssselet
    */
    #define STENCILA_LOCAL(css,xpath) BOOST_CHECK_EQUAL(CssToXPath(css),xpath);
/*
    STENCILA_LOCAL("div","")
    STENCILA_LOCAL("div.a","")
    STENCILA_LOCAL("div#a","")
    STENCILA_LOCAL("div[class]","")
    STENCILA_LOCAL("div[class=a]","")
    STENCILA_LOCAL("div[class~=a]","")
    STENCILA_LOCAL("div[class|=a]","")
    STENCILA_LOCAL("div[class=a].b#c","")
    STENCILA_LOCAL("div p","")
    STENCILA_LOCAL("div>p","")
    STENCILA_LOCAL("div > p","")
    STENCILA_LOCAL("div>p a","")
    STENCILA_LOCAL("div+a","")
    STENCILA_LOCAL("div+a+i","")
    STENCILA_LOCAL("div~a","")
    STENCILA_LOCAL("div, p,a","")
*/
    #undef STENCILA_LOCAL
}

BOOST_AUTO_TEST_CASE(select_css){
    //Selects node using CSS selector syntax
    auto node = doc.one("div.ab");
    BOOST_CHECK_EQUAL(node.child_value(),"text");
    BOOST_CHECK_EQUAL(node.attribute("data-print").value(),"x");
    
    auto nodes = doc.all("div");
    BOOST_CHECK_EQUAL(nodes.size(),(unsigned int)3);
}

BOOST_AUTO_TEST_CASE(node_append){
    //Adds a node with text child
    //Just an element
    doc.append("div");
    //An element with some text
    doc.append("div","hello");
    //An element with some attributes
    doc.append("div",{{"class","ba"},{"data-overidden","false"}});
    //An element with some attributes and some text
    doc.append("div",{{"class","bb"}},"hello");
}

BOOST_AUTO_TEST_SUITE_END()
