#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>

BOOST_AUTO_TEST_SUITE(stencil_cila)

using namespace Stencila;

// Some checking functions and macros
// Macros are used so that Boost::Unit reports lines number
// of failed chaks properly

/**
 * Check Cila to HTML
 */
std::string html(const std::string& cila){
    Stencil s;
    s.cila(cila);
    std::string html = s.html();
    boost::trim(html);
    return html;
}
#define HTML_(_CILA,_HTML) BOOST_CHECK_EQUAL(html(_CILA),_HTML);

/**
 * Check HTML to Cila
 */
std::string cila(const std::string& html){
    Stencil s;
    s.html(html);
    std::string cila = s.cila();
    boost::trim(cila);
    return cila;
}
#define CILA_(_HTML,_CILA) BOOST_CHECK_EQUAL(cila(_HTML),_CILA);

/**
 * Check Cila to Cila
 */
std::string echo(const std::string& in){
    Stencil s;
    s.cila(in);
    std::string out = s.cila();
    boost::trim(out);
    return out;
}
#define ECHO_(_CILA) BOOST_CHECK_EQUAL(echo(_CILA),_CILA);

// Check Cila to Cila, allowing for differences in input/output
#define BACK_(_IN,_OUT) BOOST_CHECK_EQUAL(echo(_IN),_OUT);

BOOST_AUTO_TEST_CASE(a){
    ECHO_("")

    ECHO_("ul\n\tli\n\tli")

}

BOOST_AUTO_TEST_CASE(shorthand){
    HTML_(".class",R"(<div class="class" />)")
    CILA_(R"(<div class="class" />)",".class")
    ECHO_(".class")

    ECHO_("#id")

    //ECHO_("[attr-a=1]")

    ECHO_(".class#id")
    ECHO_("#id.class")
}

BOOST_AUTO_TEST_CASE(directive_with){
    HTML_("with what",R"(<div data-with="what" />)")
    CILA_(R"(<div data-with="what" />)","with what")
    ECHO_("with what")

    ECHO_("section!with what")
}

BOOST_AUTO_TEST_CASE(directive_for){
    HTML_("for item in items",R"(<div data-for="item:items" />)")
}


BOOST_AUTO_TEST_CASE(cila_get){
    Stencil s;
    s.html(R"(
        <div data-if="1">
            <p>One</p>
        </div>
        <section data-elif="0">
            <p>None</p>
        </section>
        <div data-else>
            <p>None</p>
        </div>

        <ul id="a">
            <li id="a1" class="A1" data-off="true" data-lock="true" data-index="3" data-if="p<0.05">some text</li>
            <li id="a2">Yo!</li>
        </ul>
    )");
    std::cout<<s.cila();
}

BOOST_AUTO_TEST_CASE(cila_set){
    Stencil s;
    std::string cila = R"(
div#myid.myclass[foo="bar"]
    Some text in the div

    This should be a paragraph

    ul!for item in items
        li!text item

    py
        a = 1
        print a

    r
        a <-1
        print(a)

    `e = mc^2`

    )";
    s.cila(cila);
    std::cout<<cila<<"\n"<<s.html()<<std::endl;
}


#undef HTML_
#undef CILA_
#undef ECHO_
#undef BACK_

BOOST_AUTO_TEST_SUITE_END()

