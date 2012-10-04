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

#include <stencila/formats/xml.hpp>

using namespace Stencila::Formats::Xml;

struct formatsXmlFixture {

    Document doc;
    
    formatsXmlFixture(void){
        doc.load(
            "<div class='class-a'>"
            "   <div class='class-a-a'/>"
            "   <div class='class-a-b' data-print='x'>text</div>"
            "</div>"
        );
    }    
};

BOOST_FIXTURE_TEST_SUITE(formats_xml,formatsXmlFixture)

BOOST_AUTO_TEST_CASE(select_next_sibling){
    //Selects node using next_sibling
    auto node = doc.child("div").child("div").next_sibling();
    BOOST_CHECK_EQUAL(node.child_value(),"text");
    BOOST_CHECK_EQUAL(node.attribute("data-print").value(),"x");
}

BOOST_AUTO_TEST_CASE(select_xpath){
    //Selects node using Xpath
    auto node = doc.select_single_node("//div[@class='class-a-b']").node();
    BOOST_CHECK_EQUAL(node.child_value(),"text");
    BOOST_CHECK_EQUAL(node.attribute("data-print").value(),"x");
}

BOOST_AUTO_TEST_CASE(append_to){
    //Adds a node with text child
    //Just an element
    doc.append_to(doc,"div");
    //An element with some text
    doc.append_to(doc,"div","hello");
    //An element with some attributes
    doc.append_to(doc,"div",{{"class","b-a"},{"data-overidden","false"}});
    //An element with some attributes and some text
    doc.append_to(doc,"div",{{"class","b-b"}},"hello");
}

BOOST_AUTO_TEST_SUITE_END()
