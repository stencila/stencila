#include <boost/test/unit_test.hpp>

#include <stencila/xml.hpp>

BOOST_AUTO_TEST_SUITE(xml)

using namespace Stencila::Xml;

BOOST_AUTO_TEST_CASE(attributes){
    Document doc;
    Node div = doc.append("div");

    BOOST_CHECK_EQUAL(div.attr("class"),"");

    div.attr("class","foo");
    BOOST_CHECK_EQUAL(div.attr("class"),"foo");

    div.concat("class","bar");
    BOOST_CHECK_EQUAL(div.attr("class"),"foo bar");

    div.erase("class");
    BOOST_CHECK_EQUAL(div.attr("class"),"");
}

BOOST_AUTO_TEST_CASE(text){
    Document doc;
    doc.text("42");
    BOOST_CHECK_EQUAL(doc.text(),"42");
    doc.text("");
    BOOST_CHECK_EQUAL(doc.text(),"");
}

BOOST_AUTO_TEST_CASE(append){
    Document doc;

    doc.append("div");
    BOOST_CHECK(doc.find("div"));

    doc.append("span","Don't panic");
    BOOST_CHECK_EQUAL(doc.find("span").text(),"Don't panic");

    doc.append("div",{{"class","a"},{"data-ford","prefect"}});
    BOOST_CHECK(doc.find("div","class","a"));
    BOOST_CHECK(doc.find("div","data-ford","prefect"));
    
    doc.append("div",{{"class","b"}},"Don't panic");
    BOOST_CHECK_EQUAL(doc.find("div","class","b").text(),"Don't panic");

    doc.append("div",{{"class","c"}}).append_text("How many roads must a man walk down?");
    BOOST_CHECK_EQUAL(doc.find("div","class","c").text(),"How many roads must a man walk down?");

    {
        Node node = doc.append("div");
        node.append_cdata("answer = (1<2)*42");
        BOOST_CHECK_EQUAL(node.dump(),"<div><![CDATA[answer = (1<2)*42]]></div>");
    }

    {
        Node node = doc.append("div");
        node.append_comment("Isn't it enough to see that a garden is beautiful without having to believe that there are fairies at the bottom of it too?");
        BOOST_CHECK_EQUAL(node.dump(),"<div><!--Isn't it enough to see that a garden is beautiful without having to believe that there are fairies at the bottom of it too?--></div>");
    }

    doc.append_xml("<div class=\"d\"><div class=\"e\">E</div></div>");
    BOOST_CHECK_EQUAL(doc.find("div","class","d").find("div","class","e").text(),"E");
    
}

BOOST_AUTO_TEST_CASE(remove){
    Document doc;

    Node node = doc.append("div");
    BOOST_CHECK(doc.find("div"));
    doc.remove(node);
    BOOST_CHECK(not doc.find("div"));
}

BOOST_AUTO_TEST_CASE(clear){
    Document doc;
    doc.load("<body id=\"universe\"><p id=\"slartybarfast\"></p></body>");
    BOOST_CHECK(doc.find("body","id","universe"));
    BOOST_CHECK(doc.find("p","id","slartybarfast"));
    doc.clear();
    BOOST_CHECK(not doc.find("body","id","universe"));
    BOOST_CHECK(not doc.find("p","id","slartybarfast"));
}

/*
 * Test the translation of CSS selectors to XPath
 * 
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

/*
 * Test CSS selectors
 */
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

    #define CHECK(selector,result) BOOST_CHECK_EQUAL(doc.select(selector).text(),result);

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

BOOST_AUTO_TEST_CASE(sanitize){

    Document doc(R"(
        <p class="a">Foo</p>
        <script class="b" id="gives-bad-advice">alert("Panic!")</script>
        <div class="c" foo="bar" />
        <div>
            <div>
                <div>
                    <div>
                        <p>42</p>
                        <br />
                        <img class="d" href="javascript:alert('Nested badness');" />
                    </div>
                </div>
            </div>
        </div>
    )");

    BOOST_CHECK(doc.select("p.a"));
    BOOST_CHECK(doc.select("script.b"));
    BOOST_CHECK(doc.select("div.c[foo]"));
    BOOST_CHECK(doc.select("img.d"));

    doc.sanitize({
        {"p",{"class"}},
        {"div",{"class"}}
    });

    BOOST_CHECK(doc.select("p.a"));
    BOOST_CHECK(!doc.select("script"));
    BOOST_CHECK(doc.select("div.c"));
    BOOST_CHECK(!doc.select("div.c[foo]"));
    BOOST_CHECK(!doc.select("img.d"));
}

BOOST_AUTO_TEST_CASE(load_dump){
    Document doc;
    std::string content = "<div class=\"foo\">The ships hung in the sky in much the same way that bricks don't.</div>";
    doc.load(content);
    BOOST_CHECK_EQUAL(doc.dump(),content);
}

BOOST_AUTO_TEST_CASE(write_read){
    Document doc;
    std::string content = "<div class=\"foo\">The ships hung in the sky in much the same way that bricks don't.</div>";
    doc.load(content);
    auto tempfile = "/tmp/"+boost::filesystem::unique_path().string();
    doc.write(tempfile);
    doc.read(tempfile);
    BOOST_CHECK_EQUAL(doc.dump(),content);
}

BOOST_AUTO_TEST_SUITE_END()
