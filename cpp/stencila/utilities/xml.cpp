#include <boost/xpressive/xpressive_static.hpp>
#include <boost/lexical_cast.hpp>

#include <stencila/exception.hpp>
#include <stencila/utilities/xml.hpp>

namespace Stencila {
namespace Utilities {
namespace Xml {

namespace {

/*! @{

CSS selector grammar

This is a partial implementation of the grammar described in the [W3C Recommendation](http://www.w3.org/TR/css3-selectors/#w3cselgrammar)

Some of the things that are not implemented or not fully implemented
  * identifiers and strings (unicode, escape characters etc not dealt with)
  * pseudo-element ('::') 
  * pseudo-class (':')
  * negation ('not(..)')
  * namespaces ('foo|bar')

Translate a single node of a CSS selector syntax tree into an XPath selector

There are several resources that describe how to convert CSS selectors to XPath selectors (
 [e.g.1](http://www.a-basketful-of-papayas.net/2010/04/css-selectors-and-xpath-expressions.html)
 [e.g.2](http://hakre.wordpress.com/2012/03/18/css-selector-to-xpath-conversion/)
 [e.g.3](http://plasmasturm.org/log/444/)
). An actively developed implementation is the [`cssselect` module for Python](http://packages.python.org/cssselect)
and that has been used here as the primary source for how to do conversion. 
In particular, the [web app of cssselect)[http://css2xpath.appspot.com/] is a useful place for checking how to do translations.

@todo Performance could be improved by writing specific translate functions
when it is know what type a child node is. This would prevent having to
navigate the if else tree below for many cases.

*/

using namespace boost::xpressive;

sregex identifier      = +(_w|'-');

sregex string          = ('\"' >> (s1=*(~as_xpr('\"'))) >> '\"')
                       | ('\'' >> (s1=*(~as_xpr('\''))) >> '\'');

sregex element         = identifier|'*';
    
sregex attr_class      = '.' >> identifier;
sregex attr_id         = '#' >> identifier;
sregex attr_exists     = ('['  >> *space >> identifier >> *space >> ']' );
sregex attr_comparison = as_xpr("=") | "~=" | "|=" | "^=" | "$=" | "*=";
sregex attr_compare    = ('['  >> *space >> identifier >> *space >> attr_comparison >> *space >> (identifier|string) >> *space >> ']');
sregex attr            = attr_class | attr_id | attr_exists | attr_compare;
    
sregex selector        = (element >> *attr)| (+attr);

sregex descendant      = +space;
sregex child           = *space >> '>' >> *space;
sregex adjacent_sibling= *space >> '+' >> *space;
sregex general_sibling = *space >> '~' >> *space;
    
sregex selectors       = selector>>!((descendant | child | adjacent_sibling | general_sibling)>>by_ref(selectors));
    
sregex group           = selectors >> *(*space >> ',' >> *space >> selectors);
    
// Parse a CSS selector into a syntax tree
smatch parse(const std::string& selector){
    smatch tree;
    bool matched = regex_search(selector,tree,group);
    //If no match, or not fully matched, then report error
    if(matched){
        std::string match = tree.str(0);
        if(match.length()!=selector.length()){
            std::string error = selector.substr(match.length());
            STENCILA_THROW(Exception,"syntax error in: "+error);
        }
    } else {
        STENCILA_THROW(Exception,"syntax error");
    }
    return tree;
}

// Translate the CSS syteax tree into XPath
std::string translate(const smatch& node,bool adjacent=false) {
    const void* id = node.regex_id();
    if(id==attr_id.regex_id()){
        std::string id = node.str(0);
        //"#id" to "id"
        id.erase(0,1);
        return "@id='" + id + "'";
    }
    else if(id==attr_class.regex_id()){
        std::string klass = node.str(0);
        //".class" to "class"
        klass.erase(0,1);
        return "@class and contains(concat(' ',normalize-space(@class),' '),' " + klass + " ')";
    }
    else if(id==attr_exists.regex_id()){
        std::string attr = node.nested_results().begin()->str(0);
        return "@"+attr+"";
    }
    else if(id==attr_compare.regex_id()){
        auto child = node.nested_results().begin();
        std::string name = child->str(0);
        std::string op = (++child)->str(0);

        auto value_node = ++child;
        std::string value;
        // If the value is a string then (i.e surrunded by quotes) then extract the
        // string contents...
        if(value_node->regex_id()==string.regex_id()) value = value_node->str(1);
        // otherwise if it is just an identifier then use that
        else value = value_node->str(0);

        if(op=="="){
            return "@" + name + "='" + value + "'";
        }
        else if(op=="~="){
            return "@" + name + " and contains(concat(' ',normalize-space(@" + name + "),' '),' " + value + " ')";
        }
        else if(op=="|="){
            return "@" + name + " and (@" + name + "='" + value + "' or starts-with(@" + name + ",'" + value + "-'))";
        }
        else if(op=="^="){
            return "@" + name + " and starts-with(@" + name + ",'" + value + "')";
        }
        else if(op=="$="){
            return "@" + name + " and substring(@" + name + ",string-length(@" + name + ")-" + 
                // XPath's substring function uses 1-based indexing so use length-1
                boost::lexical_cast<std::string>(value.length()-1) + 
                ")='" + value + "'";
        }
        else if(op=="*="){
            return "@" + name + " and contains(@" + name + ",'" + value + "')";
        }
        return "error";
    }
    else if(id==selector.regex_id()){
        auto children = node.nested_results();
        auto attr = children.begin();
        int attrs = children.size();
        //Determine if first child node is universal (*) or not
        //If not then attrs start at second child
        const void* id = attr->regex_id();
        std::string name;
        if(id==element.regex_id()){
            name = attr->str(0);
            ++attr;
            attrs -= 1;
        } else {
            name = "*";
        }
        //Iterate through attributes
        std::string attrs_xpath;
        int index = 1;
        for(;attr!=children.end();attr++){
            attrs_xpath += translate(*attr);
            if(index<attrs) attrs_xpath += " and ";
            index++;
        }
        
        std::string xpath;
        //If this is the child of an adjacent selectors node then
        //the generated Xpath needs to be different
        if(adjacent){
            xpath = "*[name()='" + name + "' and (position()=1)";
            if(attrs>0) xpath += " and " + attrs_xpath;
            xpath += ']';
        } else {
            xpath = name;
            if(attrs>0) xpath += '[' + attrs_xpath + ']';
        }
        return xpath;
    }
    else if(id==selectors.regex_id()){
        //Determine if a relation is involved
        auto children = node.nested_results();
        if(children.size()==1){
            //No relation, just a simple selector
            return translate(*children.begin(),adjacent);
        } else {
            //Determine the type of the relation (the second child of the node)
            auto branch = children.begin();
            auto left = branch;
            auto relation = ++branch;
            auto right = ++branch;
            const void* id = relation->regex_id();
            if(id==descendant.regex_id()){
                return translate(*left,adjacent)+"/descendant::"+translate(*right);
            }
            else if(id==child.regex_id()){
                return translate(*left,adjacent)+"/"+translate(*right);
            }
            else if(id==adjacent_sibling.regex_id()){
                return translate(*left,adjacent)+"/following-sibling::"+ translate(*right,true);
            }
            else if(id==general_sibling.regex_id()){
                return translate(*left,adjacent)+"/following-sibling::"+ translate(*right);
            }
            return "error";
        }
    }
    else if(id==group.regex_id()){
        //Root of sytax tree.
        std::string xpath = "descendant-or-self::";
        //Separate selectors using |
        auto children = node.nested_results();
        int n = children.size();
        int index = 1;
        for(auto i=children.begin();i!=children.end();i++){
            xpath += translate(*i);
            if(index<n) xpath += " | ";
            index++;
        }
        return xpath;
    }
    else {
        //Default is to translate each child node
        std::string xpath;
        for(auto i=node.nested_results().begin();i!=node.nested_results().end();i++){
            xpath += translate(*i);
        }
        return xpath;
    }
}

}// anonymous namespace

std::string Node::xpath(const std::string& selector) {
    return translate(parse(selector));
}

}
}
}