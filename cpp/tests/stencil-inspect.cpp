#include <iostream>

#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE()

BOOST_AUTO_TEST_CASE(parse_code_0){
    try {
        Stencil::parse_code("a a a");
    } catch(const Exception& exc){
        BOOST_CHECK_EQUAL(exc.message(),"Syntax error in code directive attribute <a a a>");
    }  
}

BOOST_AUTO_TEST_CASE(parse_code_1){
    auto code = Stencil::parse_code("py");
    BOOST_CHECK_EQUAL(code.contexts.size(),1);
    BOOST_CHECK_EQUAL(code.contexts[0],"py");
}

BOOST_AUTO_TEST_CASE(parse_code_2){
    auto code = Stencil::parse_code("r,py");
    BOOST_CHECK_EQUAL(code.contexts.size(),2);
    BOOST_CHECK_EQUAL(code.contexts[0],"r");
    BOOST_CHECK_EQUAL(code.contexts[1],"py");
}

BOOST_AUTO_TEST_CASE(parse_code_3){
    auto code = Stencil::parse_code("r,   py");
    BOOST_CHECK_EQUAL(code.contexts.size(),2);
    BOOST_CHECK_EQUAL(code.contexts[0],"r");
    BOOST_CHECK_EQUAL(code.contexts[1],"py");
}

BOOST_AUTO_TEST_CASE(parse_code_4){
    try {
        Stencil::parse_code("r,bf");
    } catch(const Exception& exc){
        BOOST_CHECK_EQUAL(exc.message(),"Context type <bf> is not valid");
    }  
}

BOOST_AUTO_TEST_CASE(parse_code_5){
    auto code = Stencil::parse_code("r text");
    BOOST_CHECK_EQUAL(code.format,"text");
}

BOOST_AUTO_TEST_CASE(parse_code_6){
    auto code = Stencil::parse_code("r png");
    BOOST_CHECK_EQUAL(code.format,"png");
}

BOOST_AUTO_TEST_CASE(parse_code_7){
    auto code = Stencil::parse_code("r svg");
    BOOST_CHECK_EQUAL(code.format,"svg");
}

BOOST_AUTO_TEST_CASE(parse_code_8){
    try {
        Stencil::parse_code("r gnp");
    } catch(const Exception& exc){
        BOOST_CHECK_EQUAL(exc.message(),"Format <gnp> is not valid");
    }  
}

BOOST_AUTO_TEST_CASE(parse_code_9){
    auto code = Stencil::parse_code("py,r png 4.2x8.4");
    BOOST_CHECK_EQUAL(code.contexts[0],"py");
    BOOST_CHECK_EQUAL(code.contexts[1],"r");
    BOOST_CHECK_EQUAL(code.format,"png");
    BOOST_CHECK_EQUAL(code.width,"4.2");
    BOOST_CHECK_EQUAL(code.height,"8.4");
    BOOST_CHECK_EQUAL(code.units,"");
}

BOOST_AUTO_TEST_CASE(parse_code_10){
    try {
        Stencil::parse_code("r png 10x10km");
    } catch(const Exception& exc){
        BOOST_CHECK_EQUAL(exc.message(),"Size units <km> is not valid");
    }  
}

BOOST_AUTO_TEST_CASE(parse_for_0){
    try {
        Stencil::parse_for("foo bar");
    } catch(const Exception& exc){
        BOOST_CHECK_EQUAL(exc.message(),"Syntax error in for directive attribute <foo bar>");
    }  
}

BOOST_AUTO_TEST_CASE(parse_for_1){
    auto forr = Stencil::parse_for("foo in bar");
    BOOST_CHECK_EQUAL(forr.name,"foo");
    BOOST_CHECK_EQUAL(forr.expr,"bar");  
}

BOOST_AUTO_TEST_CASE(parse_for_2){
    auto forr = Stencil::parse_for("foo     in       bar");
    BOOST_CHECK_EQUAL(forr.name,"foo");
    BOOST_CHECK_EQUAL(forr.expr,"bar");  
}

BOOST_AUTO_TEST_CASE(include_parse_1){
    Stencil::Include inc;
    inc.parse("includee");
    BOOST_CHECK_EQUAL(inc.includee,"includee"); 
}

BOOST_AUTO_TEST_CASE(include_parse_2){
    Stencil::Include inc;
    inc.parse("includee version 0.1");
    BOOST_CHECK_EQUAL(inc.includee,"includee"); 
    BOOST_CHECK_EQUAL(inc.version,"0.1");
}

BOOST_AUTO_TEST_CASE(include_parse_3){
    Stencil::Include inc;
    inc.parse("includee select #id");
    BOOST_CHECK_EQUAL(inc.includee,"includee"); 
    BOOST_CHECK_EQUAL(inc.select,"#id");
}

BOOST_AUTO_TEST_SUITE_END()
