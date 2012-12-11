/*
Copyright (c) 2012, Stencila Ltd
Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

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
    NodeAppend(doc,"div");
    //An element with some text
    NodeAppend(doc,"div","hello");
    //An element with some attributes
    NodeAppend(doc,"div",{{"class","ba"},{"data-overidden","false"}});
    //An element with some attributes and some text
    NodeAppend(doc,"div",{{"class","bb"}},"hello");
}

BOOST_AUTO_TEST_SUITE_END()
