#include <iostream>

#ifdef STENCILA_TEST_SINGLE
    #define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/utilities/html.hpp>

BOOST_AUTO_TEST_SUITE(html)

using namespace Stencila::Utilities::Html;

BOOST_AUTO_TEST_CASE(load){

    BOOST_CHECK_EQUAL(
    	Document().dump(),
    	"<!DOCTYPE html><html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title /><meta http-equiv=\"Content-Type\" content=\"application/xhtml+xml\" /><meta charset=\"UTF-8\" /></head><body /></html>"
    );

    #define CHECK(in,out) BOOST_CHECK_EQUAL(Document(in).find("body").dump_children(),out);

    CHECK(
        "<h2>subheading</h3>",
        "<h2>subheading</h2>"
    )

    // As of commit 0cf6d99843 tidy-html5 did not recognise <main> tags (https://github.com/w3c/tidy-html5/issues/82)
    // We add a patch to fix that. This check tests that <main> is indeed recognised.
    CHECK(
        "<main id=\"content\">content</main>",
        "<main id=\"content\">content</main>"
    )

    #undef CHECK
}

BOOST_AUTO_TEST_SUITE_END()
