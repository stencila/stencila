#ifdef STENCILA_TEST_SINGLE
	#define BOOST_TEST_MODULE tests
#endif
#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>

BOOST_AUTO_TEST_SUITE(stencil_cila)

using namespace Stencila;

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

BOOST_AUTO_TEST_SUITE_END()

