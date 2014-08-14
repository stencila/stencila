#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/xpressive/xpressive_static.hpp>
#include <boost/xpressive/regex_compiler.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

/**
 * Define stencil whitelist
 *
 * Note that below all STENCILA_STENCIL_TAGS are allowed to have all STENCILA_STENCIL_ATTRS
 * but that can be overidden by placing an item before the call to BOOST_PP_SEQ_FOR_EACH
 */
Xml::Whitelist Stencil::whitelist = {
    {"img",{"src",STENCILA_STENCIL_ATTRS}},

    // Define an item macro, run it through BOOST_PP_SEQ_FOR_EACH, and then undef it
    #define STENCILA_WHITELIST_ITEM_(repeater,separator,tag) {BOOST_PP_STRINGIZE(tag),{STENCILA_STENCIL_ATTRS}},
    BOOST_PP_SEQ_FOR_EACH(STENCILA_WHITELIST_ITEM_, ,STENCILA_STENCIL_TAGS)
    #undef STENCILA_WHITELIST_ITEM_

    {} // Required due to trailing comma in STENCILA_WHITELIST_ITEM_
};

}
