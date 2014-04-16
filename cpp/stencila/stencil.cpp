#include <stencila/stencil.hpp>

namespace Stencila {

std::stack<Stencil::Node> Stencil::embed_parents_;

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

/**
 * A macro designed to be called by BOOST_PP_SEQ_FOR_EACH to define free functions for each of STENCILA_STENCIL_TAGS
 * Note that the use of BOOST_PP_STRINGIZE is required instead of the stringizing operator (#) which prevents arguments from expanding
 */
#define STENCILA_LOCAL(repeater,separator,tag)\
    Html::Node tag(void){                                                                              return Stencil::element(BOOST_PP_STRINGIZE(tag));                                 } \
    Html::Node tag(const std::string& text){                                                           return Stencil::element(BOOST_PP_STRINGIZE(tag),text);                            } \
    Html::Node tag(const Stencil::AttributeList& attributes, const std::string& text=""){              return Stencil::element(BOOST_PP_STRINGIZE(tag),attributes,text);                 } \
    Html::Node tag(void(*inner)(void)){                                                                return Stencil::element(BOOST_PP_STRINGIZE(tag),Stencil::AttributeList(),inner);  } \
    template<typename... Args> Html::Node tag(const Stencil::AttributeList& attributes,Args... args){  return Stencil::element(BOOST_PP_STRINGIZE(tag),attributes,args...);              } \
    template<typename... Args> Html::Node tag(Args... args){                                           return Stencil::element(BOOST_PP_STRINGIZE(tag),args...);                         }
BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, ,STENCILA_STENCIL_TAGS)
#undef STENCILA_LOCAL

void if_(const std::string& expression, const std::string& tag="div"){
    Stencil::start(tag,Stencil::AttributeList{{"data-if",expression}});
}

void for_(const std::string& tag="div"){
    Stencil::element(tag,"data-for");
}

void end(void){
    Stencil::finish();
}

void include(const std::string& tag="div"){
    Stencil::element(tag,"data-include");
}

}
