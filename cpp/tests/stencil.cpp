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

#include <stencila/stencil.hpp>
#include <stencila/simple-context.hpp>

using namespace Stencila;

struct stencil1Fixture {
    
    Stencil stencil1,stencil2;
    
    stencil1Fixture(void){
        
        stencil1.from_html(
            "<div data-alias='stencil1'/>"
        );
        stencil1.id();
        
        stencil2.from_html(
            "<div data-include='id://"+stencil1.id()+"'/>"
        );
    }    
};

BOOST_FIXTURE_TEST_SUITE(stencil1,stencil1Fixture)

BOOST_AUTO_TEST_CASE(create_empty){
    Stencil s;
    BOOST_CHECK_EQUAL(s.body(),"");
}

BOOST_AUTO_TEST_CASE(id){
    Stencil s1;
    Stencil* s2 = Component<>::obtain<Stencil>(s1.id());
    BOOST_CHECK(s2!=0);
    BOOST_CHECK_EQUAL(s1.id(),s2->id());
}

BOOST_AUTO_TEST_CASE(create_html_fragment){
    Stencil s("html://<p>Hello world</p>");
    BOOST_CHECK_EQUAL(s.body(),"<p>Hello world</p>");
}

BOOST_AUTO_TEST_CASE(create_html_page){
    Stencil s(R"(html://
    <html>
        <head>
            <meta name="description" content="Says hello to the world">
            <meta name="keywords" content="greeting, salutation">
        </head>
        <body>
            <p>Hello world!</p>
        </body>
    </html>
    )");
    
    {
        std::vector<std::string> got = s.keywords();
        std::vector<std::string> exp = {"greeting","salutation"};
        BOOST_CHECK_EQUAL_COLLECTIONS(got.begin(),got.end(),exp.begin(),exp.end());
    }
    BOOST_CHECK_EQUAL(s.body(),"<p>Hello world!</p>");
}

BOOST_AUTO_TEST_CASE(render){
    SimpleContext context;
    stencil2.render(context);
}

BOOST_AUTO_TEST_CASE(render_include){
    Stencil stencil(R"(html://
        <div data-include="id://stencil21387598">
            <div data-replace="#an-id"/>
            <div data-before="#an-id"/>
            <div data-after="#an-id"/>
            <div data-prepend="#an-id"/>
            <div data-append="#an-id"/>
        </div>
    )");
    SimpleContext context;
    stencil.render(context);
    //std::cout<<stencil.dump()<<"\n\n";
}

void stem_html(std::string stem,std::string html) {
    Stencil s;
    s.from_stem(stem);
    std::string got = s.body();
    if(got!=html){
        BOOST_ERROR("\n\tstem: "+stem+"\n\texpected: "+html+"\n\tgot     : "+got+"\n\ttree:\n"+Stencil::stem_print(stem));
    }
}

BOOST_AUTO_TEST_CASE(stem_1){    
    //Plain old text
    stem_html("The quick brown fox","The quick brown fox");
    stem_html("Divide","Divide");
    
    //Plain old HTML elements
    stem_html("div","<div />");
    stem_html("p","<p />");
    stem_html("section","<section />");
    
    //HTML elements followed by some text
    stem_html("p This is my short paragraph","<p>This is my short paragraph</p>");
    stem_html("span            This is my span","<span>This is my span</span>");
    
    //HTML elements with attributes
    stem_html("div.klass","<div class=\"klass\" />");
    stem_html("div.klass1.klass2","<div class=\"klass1 klass2\" />");
    stem_html("div.klass-a.klass-b.klass-c-d","<div class=\"klass-a klass-b klass-c-d\" />");
    
    stem_html("div#ident","<div id=\"ident\" />");
    stem_html("div#ident-a","<div id=\"ident-a\" />");
    stem_html("div#ident.klass","<div id=\"ident\" class=\"klass\" />");

    stem_html("div attr=\"1\"","<div attr=\"1\" />");
    stem_html("div[attr=\"1\"]","<div attr=\"1\" />");
    stem_html("div attr1=\"1\" attr2=\"2\"","<div attr1=\"1\" attr2=\"2\" />");
    stem_html("div[attr1=\"1\"][attr2=\"2\"]","<div attr1=\"1\" attr2=\"2\" />");
    
    stem_html("a[href=\"http://stenci.la\"] Stencila","<a href=\"http://stenci.la\">Stencila</a>");
    
    //HTML elements with just attribute (default to div)
    stem_html(".klass","<div class=\"klass\" />");
    stem_html(".klass1.klass2","<div class=\"klass1 klass2\" />");
    stem_html("#ident","<div id=\"ident\" />");
    stem_html("[attr=\"1\"]","<div attr=\"1\" />");
    stem_html("attr=\"1\"","attr=\"1\""); //Note that this is supposed to be treated as a text node
    
    //HTML elements nested 
    stem_html(R"(
div
  p
    span Hello world!
)","<div><p><span>Hello world!</span></p></div>");

    //Directives
    
    //text
    stem_html("text variable",R"(<div data-text="variable" />)");
    stem_html("span!text variable",R"(<span data-text="variable" />)");
    
    stem_html("|variable|",R"(<span data-text="variable" />)");
    stem_html("Ab cdefg |variable| hijk",R"(Ab cdefg <span data-text="variable" /> hijk)");
    stem_html("Ab cdefg |variable1| hijk |variable2|",R"(Ab cdefg <span data-text="variable1" /> hijk <span data-text="variable2" />)");
    stem_html("Ab cdefg a|variable1| hijk b|variable2|",R"(Ab cdefg <a data-text="variable1" /> hijk <b data-text="variable2" />)");
    
    //r , py
    stem_html(R"(
r
    a <- 42
)",
R"(<script type="text/r">#<![CDATA[
    a <- 42
#]]></script>)");

    stem_html(R"(
r
    a <- 42
)",
R"(<script type="text/r">#<![CDATA[
    a <- 42
#]]></script>)");

    //With Python how should indentation be handled?
    stem_html(R"(
py
    a = 42
    if a>1:
        b = 2
    else:
        b = 1
)",
R"(<script type="text/py">#<![CDATA[
    a = 42
    if a>1:
        b = 2
    else:
        b = 1
#]]></script>)");

    //if
    stem_html(R"(
if true
    p 1
elif false
    p 2
else
    p 3
)",
R"(<div data-if="true"><p>1</p></div><div data-elif="false"><p>2</p></div><div data-else=""><p>3</p></div>)");

    //switch
    stem_html(R"(
switch a
    value 3.14
        Pi
    value 42
        The answer
    default
        A number
)",
R"(<div data-switch="a"><div data-value="3.14">Pi</div><div data-value="42">The answer</div><div data-default="">A number</div></div>)");

    //for
    stem_html(R"(
for item in items
    text item
)",
R"(<div data-for="item:items"><div data-text="item" /></div>)");
    stem_html(R"(
ul!for item in items
    li!text item
)",
R"(<ul data-for="item:items"><li data-text="item" /></ul>)");

    //include
    stem_html(R"(
include another_stencil #header
    replace #brand-name
        Acme
    before #x
        .my_class Hello
)",
R"(<div data-include="another_stencil" data-select="#header"><div data-replace="#brand-name">Acme</div><div data-before="#x"><div class="my_class">Hello</div></div></div>)");

    //Comment
    stem_html("// A comment",R"(<!-- A comment -->)");

    stem_html(R"(
    // start
        line1
        line2
)",
R"(<!-- start
        line1
        line2
 -->)");

}

BOOST_AUTO_TEST_CASE(create_stem_string){
    Stencil s("stem://.klass#ident");
    BOOST_CHECK_EQUAL(s.body(),"<div class=\"klass\" id=\"ident\" />");
}
BOOST_AUTO_TEST_CASE(create_stem_file){
    Stencil s("file://inputs/a.stem");
    BOOST_CHECK_EQUAL(s.body(),"<div><ul><li /></ul></div>");
}

BOOST_AUTO_TEST_SUITE_END()
