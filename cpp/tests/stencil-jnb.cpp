#include <boost/test/unit_test.hpp>

#include <stencila/stencil.hpp>
using namespace Stencila;

BOOST_AUTO_TEST_SUITE(stencil_jnb)

BOOST_AUTO_TEST_CASE(to_from){
  Stencil s;

  for(auto pair : std::vector<std::array<std::string,2>>{
    {R"({
      "metadata" : {
        "kernel_info": {
            "name" : "the name of the kernel"
        },
        "language_info": {
            "name" : "the programming language of the kernel",
            "version": "the version of the language",
            "codemirror_mode": "The name of the codemirror mode to use [optional]"
        }
      },
      "nbformat": 4,
      "nbformat_minor": 0,
      "cells" :[
        {
          "cell_type" : "markdown",
          "source" : "some *markdown*"
        },{
          "cell_type" : "code",
          "source" : "x <- 6 * 7"
        }
      ]
    })", 
      R"()"
    },
  }) {
    s.jnb(pair[0]);
    BOOST_CHECK_EQUAL(s.html(), pair[1]);
  }
}

BOOST_AUTO_TEST_SUITE_END()
