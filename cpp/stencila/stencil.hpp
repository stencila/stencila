/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file stencil.hpp
//! @brief Definition of class Stencil

#pragma once

#include <string>
#include <fstream>

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/date_time/posix_time/posix_time.hpp>
#include <boost/xpressive/regex_compiler.hpp>
#include <boost/filesystem.hpp>

#include <stencila/formats/xml.hpp>
namespace Xml = Stencila::Formats::Xml;

namespace Stencila {

template<class Derived>
class Context {
public:

};

class EchoContext : public Context<EchoContext> {
public:

    void set(const std::string& name, const std::string& expression){
    }

    void script(const std::string& code){
    }

    std::string text(const std::string& expression){
        return "text("+expression+")";
    }

    bool test(const std::string& expression){
        return false;
    }

    void subject(const std::string& expression){
    }

    bool match(const std::string& expression){
        return false;
    }

    void enter(void){
    }

    void enter(const std::string& expression){
    }

    void exit(void){
    }

    bool begin(const std::string& item,const std::string& items){
        return false;
    }

    bool step(void){
        return false;
    }
};

class Stencil : public Xml::Document {
private:

    std::string id_;
    std::string uri_;

    template<typename Context>
    void render_element(Xml::Node node, Context& context){
        try {
            //Check for handled element tag names
            std::string tag = node.name();
            if(tag=="script") return render_script(node,context,node.text().as_string());
            //For each attribute in this node...
            auto attrs = node.attributes();
            for(auto attr=attrs.begin();attr!=attrs.end();attr++){
                //...get the name and value of attribute
                std::string name = attr->name();
                std::string value = attr->value();
                //...use the name of the attribute to dispatch to another method
                //   Note that return is used so that only the first Stencila "data-xxx" will be 
                //   considered and that directive will determin how/if children nodes are processed
                if(name=="data-text") return render_text(node,context,value);
                else if(name=="data-if") return render_if(node,context,value);
                else if(name=="data-switch") return render_switch(node,context,value);
                else if(name=="data-for") return render_for(node,context,value);
                else if(name=="data-with") return render_with(node,context,value);
                else if(name=="data-include") return render_include(node,context,value);
            }
            //If return not yet hit then process children of this element
            render_children(node,context);
        }
        catch(std::exception& exc){
            Xml::NodeSetAttribute(node,"data-error",exc.what());
        }
        catch(...){
            Xml::NodeSetAttribute(node,"data-error","unknown error");
        }
    }
    
    template<typename Context>
    void render_children(Xml::Node node, Context& context){
        for(Xml::Node child:node.children()){
            render_element(child,context);
        }
    }

    template<typename Context>
    void render_script(Xml::Node node, Context& context, const std::string& code){
         context.script(code);
    }

    template<typename Context>
    void render_text(Xml::Node node, Context& context, const std::string& expression){
        try {
            std::string text = context.text(expression);
            node.text().set(text.c_str());
        }
        catch(std::exception& exc){
            Xml::NodeSetAttribute(node,"data-error",exc.what());
        }
        catch(...){
            Xml::NodeSetAttribute(node,"data-error","unknown error");
        }
    }

    template<typename Context>
    void render_with(Xml::Node node, Context& context, const std::string& expression){
        //Enter a new block in the context
        context.enter(expression);
        //Render all children of node within that new blockŔ
        render_children(node,context);
        //Exit the block
        context.exit();
    }

    template<typename Context>
    void render_if(Xml::Node node, Context& context, const std::string& expression){
        //Test the expression
        bool result = context.test(expression);
        //If test passes, render all children
        if(result){
            render_children(node,context);
            Xml::NodeSetAttribute(node,"data-active","true");
        }
        //If test fails, remove the data-active attribute (if it exists)
        else {
            node.remove_attribute("data-active");
        }
    }

    template<typename Context>
    void render_switch(Xml::Node node, Context& context, const std::string& expression){
        //Evaluate the expression in the context
        context.subject(expression);
        //Iterate through children to
        // (a) find first child that has an equal when
        // (b) remove data-active attribute
        Xml::Node active;
        for(Xml::Node child : node.children()){
            child.remove_attribute("data-active");
            Xml::Attribute when = Xml::NodeGetAttribute(child,"data-value");
            if(when){
                bool equal = context.match(when.value());
                if(equal){
                    active = child;
                    break;
                }
            } else if(Xml::NodeGetAttribute(child,"data-default")){
                active = child;
            }
        }
        if(active){
            //Set as active
            Xml::NodeSetAttribute(active,"data-active","true");
            //Render it
            render_element(active,context);
        }
    }

    template<typename Context>
    void render_for(Xml::Node node, Context& context, const std::string& value){
        // Get the name of item and items
        std::vector<std::string> bits;
        boost::split(bits,value,boost::is_any_of(":"));
        std::string item = bits[0];
        std::string items = bits[1];
        // Initialise the loop
        bool more = context.begin(item,items);
        // Get the first child element of this node for replication
        Xml::Node first = node.find_child(Xml::NodeIsElement);
        // Delete all other nodes
        for(Xml::Node child : node.children()){
            if(child!=first) node.remove_child(child);
        }
        int count = 1;
        while(more){
            if(count==1){
                //Render the first child
                render_element(first,context);
            } else {
                //Create a copy of the first child node
                Xml::Node copy = node.append_copy(first);
                //Render the copy
                render_element(copy,context);
            }
            //Ask context to step
            more = context.step();
            count++;
        }
    }

    template<typename Context>
    void render_include(Xml::Node node, Context& context, const std::string& identifier){

        //Remove any existing children that have been included previously
        for(Xml::Node child : node.children()){
            if(Xml::NodeHasAttribute(child,"data-included")){
                node.remove_child(child);
            }
        }

        // Get the included stencil
        Stencil source(identifier);
        Xml::Document sink;
        // Check to see if a subselection of modes is to be included
        Xml::Attribute select = Xml::NodeGetAttribute(node,"data-select");
        if(select){
            Xml::Nodes included = source.all(select.value());
            for(auto i=included.begin();i!=included.end();i++) sink.append_copy(i->node());
        }
        //Otherwise include all children
        else {
            for(auto i=source.children().begin();i!=source.children().end();i++) sink.append_copy(*i);
        }
        
        //Apply child modifiers
        std::string modifiers[] = {"replace","before","after","prepend","append"};
        enum {replace=0,before=1,after=2,prepend=3,append=4};
        for(Xml::Node child : node.children()){
            for(unsigned int modifier=0;modifier<5;modifier++){
                std::string attr_name = "data-" + modifiers[modifier];
                Xml::Attribute attr = Xml::NodeGetAttribute(child,attr_name);
                if(attr){
                    Xml::Nodes targets = sink.all(attr.value());
                    for(auto i=targets.begin();i!=targets.end();i++){
                        Xml::Node target = i->node();
                        Xml::Node copy;
                        switch(modifier){
                            case replace: 
                                copy = sink.insert_copy_before(child,target);
                                sink.remove_child(target);
                            break;
                            
                            case before:
                                copy = sink.insert_copy_before(child,target);
                            break;
                            
                            case after:
                                copy = sink.insert_copy_after(child,target);
                            break;
                            
                            case prepend:
                                copy = target.prepend_copy(child);
                            break;
                            
                            case append:
                                copy = target.append_copy(child);
                            break;
                        }
                        copy.remove_attribute(attr_name.c_str());
                    }
                    break;
                }
            }
        }

        //Append new, included children
        for(Xml::Node child : sink.children()){
            Xml::NodeSetAttribute(child,"data-included","true");
            node.append_copy(child);
        }

        //Create a new context with parameters
        //Determine if there are any node parameters so that we don't create a new context block unecessarily
        bool params = Xml::NodeHasAttribute(node,"data-param");
        if(params){
            //Enter a new anonymous block
            context.enter();
            //Map the "data-param" attributes into the context...
            auto attrs = node.attributes();
            for(auto attr=attrs.begin();attr!=attrs.end();attr++){
                // Don't try to factor out this string for name. If its not there the 
                // string comparison does not work
                std::string name = attr->name();
                if(name=="data-param"){
                    std::string value = attr->value();
                    //Get the name and value of the parameter
                    //! @todo the parsing of value should be done in a specific method with
                    //! error capture and reporting
                    std::vector<std::string> bits;
                    boost::split(bits,value,boost::is_any_of(":"));
                    std::string parameter = bits[0];
                    std::string expression = bits[1];
                    //Set the parameter in the new block
                    context.set(parameter,expression);
                }
            }
        }

        //Render the new children of this node (within the new block)
        render_children(node,context);
        
        //Exit the anonymous block if created
        if(params) context.exit();
    }

public:

    Stencil(void){
    }

    Stencil(const std::string& content){
        /*
        html://
        stem://
        
        file://
        http://
        
        id://
        find://
        */
        std::size_t found = content.find("://");
        if(found==std::string::npos) STENCILA_THROW(Exception,"Type separator (://) not found")
        std::string type = content.substr(0,found);
        std::string rest = content.substr(found+3);
        if(type=="html") html(rest);
        else if(type=="stem") stem(rest);
        else if(type=="file") file(rest);
        else if(type=="id") id(rest);
        else STENCILA_THROW(Exception,"Unrecognised type: " + type)
    }

    Stencil& html(const std::string& html){
        load(html);
        return *this;
    }

    Stencil& stem(const std::string& stem);

    Stencil& file(const std::string& path){
        std::ifstream file(path);
        std::stringstream buffer;
        buffer<<file.rdbuf();
        std::string ext = boost::filesystem::path(path).extension().string();
        if(ext==".html") {
            load(buffer.str());
        }
        else if(ext==".stem") {
            stem(buffer.str());
        } 
        else {
            STENCILA_THROW(Exception,"File extension not interpreted as a stencil:"+ext)
        }
        
        return *this;
    }

    Stencil& id(const std::string& id){
        return *this;
    }

    void identify(void){
        //Add an id if one does not yet exist
        if(id_.length()==0) id_ = boost::uuids::to_string(boost::uuids::random_generator()());
    }

    std::string id(void) const {
        return id_;
    }

    std::string uri(void) const {
        return uri_;
    }

    template<typename Context>
    Stencil& render(Context& context){
        render_element(*this,context);
        return *this;
    }
};


namespace Stem {

/*!
 Stencila markup language : a Jade/Slim/Scaml/Haml-like language for Stencil templates

 * Jade http://jade-lang.com/
 * Slim http://slim-lang.com/
 * Haml http://haml.info
 * Scaml http://scalate.fusesource.org/documentation/scaml-reference.html
 * Dmsl https://github.com/dskinner/dmsl
 * 
 * http://www.boost.org/doc/libs/1_49_0/doc/html/xpressive/user_s_guide.html
 */ 

using namespace boost::xpressive;

//! Defines the id of each sytax component for faster matching in translate function
#define ID(name) const void* name##_ = name.regex_id();


#if 0
sregex element_name = +_w;
#elif 0
/*
List of vaild HTML5 element names from 
    http://www.w3.org/TR/html-markup/elements.html
and extracted using this python script:
    import requests
    import bs4
    page = requests.get('http://www.w3.org/TR/html-markup/elements.html').text
    elems = bs4.BeautifulSoup(page).findAll('span', {'class':'element'})
    print '|'.join('"%s"'%elem.text for elem in sorted(set(elems)))

Statically compiling element name list dramatically increases compile
times (e.g. 11s to 27s) and executable sizes (e.g. 10Mb to 80Mb).
*/
sregex element_name     = as_xpr("a")|"abbr"|"address"|"area"|"article"|"aside"|"audio"|"b"|"base"|"bdi"|"bdo"|"blockquote"|"body"|"br"|"button"|
                            "canvas"|"caption"|"cite"|"code"|"col"|"colgroup"|"command"|"datalist"|"dd"|"del"|"details"|"dfn"|"div"|"dl"|"dt"|
                            "em"|"embed"|"fieldset"|"figcaption"|"figure"|"footer"|"form"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"head"|"header"|"hgroup"|"hr"|"html"|
                            "i"|"iframe"|"img"|"input"|"ins"|"kbd"|"keygen"|"label"|"legend"|"li"|"link"|"map"|"mark"|"menu"|"meta"|"meter"|"nav"|"noscript"|
                            "object"|"ol"|"optgroup"|"option"|"output"|"p"|"param"|"pre"|"progress"|"q"|"rp"|"rt"|"ruby"|"s"|"samp"|"script"|"section"|
                            "select"|"small"|"source"|"span"|"strong"|"style"|"sub"|"summary"|"sup"|"table"|"tbody"|"td"|"textarea"|"tfoot"|"th"|"thead"|
                            "time"|"title"|"tr"|"track"|"u"|"ul"|"var"|"video"|"wbr";
#elif 1
/*
Dynamically compiling element name list only slightly increases compile
times (e.g. 11s to 15s) and executable sizes (e.g. 10Mb to 13Mb).
*/
sregex element_name = sregex::compile(
                            "a|abbr|address|area|article|aside|audio|b|base|bdi|bdo|blockquote|body|br|button|"
                            "canvas|caption|cite|code|col|colgroup|command|datalist|dd|del|details|dfn|div|dl|dt|"
                            "em|embed|fieldset|figcaption|figure|footer|form|h1|h2|h3|h4|h5|h6|head|header|hgroup|hr|html|"
                            "i|iframe|img|input|ins|kbd|keygen|label|legend|li|link|map|mark|menu|meta|meter|nav|noscript|"
                            "object|ol|optgroup|option|output|p|param|pre|progress|q|rp|rt|ruby|s|samp|script|section|"
                            "select|small|source|span|strong|style|sub|summary|sup|table|tbody|td|textarea|tfoot|th|thead|"
                            "time|title|tr|track|u|ul|var|video|wbr"
);
#endif
    ID(element_name)
    
sregex inlinee_expr      = *(~(set='|'));
    ID(inlinee_expr)
    
sregex inlinee           = *element_name >> "|" >> inlinee_expr >> "|";
    ID(inlinee)
    
sregex chars            = *space>>+(~(set='|',' ','\t'))>>*space;
    ID(chars)

sregex text             = +(inlinee|chars);
    ID(text)


sregex code = sregex::compile("py|r");
    ID(code)

///////////////////

sregex expr = +_;
    ID(expr)

sregex directive_for = as_xpr("for") >> +space >> expr >> +space >> "in" >> +space >> expr;
    ID(directive_for)

///////////////////

//CSS selector
sregex css_selector = +_;
    ID(css_selector)

//Stencil identifier
sregex stencil_identifier = +_w;
    ID(stencil_identifier)

///////////////////

sregex directive_include = as_xpr("include") >> +space >> stencil_identifier >> *(+space >> css_selector);
    ID(directive_include)

///////////////////

sregex directive_modifier_name = sregex::compile("replace|before|after|prepend|append");
    ID(directive_modifier_name)

sregex directive_modifier = directive_modifier_name >> +space >> css_selector;
    ID(directive_modifier)

///////////////////

sregex directive_arg_name = sregex::compile("text|with|if|elif|switch|value");
    ID(directive_arg_name)
    
sregex directive_arg = directive_arg_name >> +space >> expr;
    ID(directive_arg)

///////////////////
    
sregex directive_noarg = sregex::compile("script|else|default");
    ID(directive_noarg)

///////////////////

sregex attr_identifier       = +(_w|'-');
    ID(attr_identifier)

sregex attr_string            = ('\"' >> *(~(set='\r','\n','\"')) >> '\"') | 
                           ('\'' >> *(~(set='\r','\n','\'')) >> '\'');
    ID(attr_string)

sregex attr_class       = '.' >> attr_identifier;
    ID(attr_class)
sregex attr_id          = '#' >> attr_identifier;
    ID(attr_id)
sregex attr_assign      = attr_identifier >> '=' >> attr_string;
    ID(attr_assign)

sregex element          = (
    (*(element_name >> "!") >> (directive_include|directive_modifier|directive_for|directive_arg|directive_noarg)) |
    (element_name >> +(+space >> attr_assign)) |
    (element_name >> *(attr_class|attr_id|'[' >> *space >> +(attr_assign>>*space) >> ']')) |
                     +(attr_class|attr_id|'[' >> *space >> +(attr_assign>>*space) >> ']')
) >> *(+space >> *text);
    ID(element)

///////////////////

sregex comment_text = *_;
    ID(comment_text)
    
sregex comment = as_xpr("//") >> comment_text;
    ID(comment)

///////////////////

sregex indent = *space;
    ID(indent)

sregex line = indent >> (comment|code|element|text);
    ID(line)

#undef ID
//! @}

std::map<const void*,std::string> rules;
inline
void initialise(void) {
    static bool initialised = false;
    if(initialised) return;

    #define MAP(name) rules[name.regex_id()] = #name;
    MAP(inlinee_expr)
    MAP(inlinee)
    MAP(chars)
    MAP(text)
    
    MAP(code)
    
    
    MAP(css_selector)
    MAP(stencil_identifier)
    MAP(directive_include)
    MAP(directive_modifier)
    
    MAP(expr)
    MAP(directive_for)
    MAP(directive_arg_name)
    MAP(directive_arg)
    MAP(directive_noarg)
    
    MAP(attr_identifier)
    MAP(attr_string)
    MAP(attr_class)
    MAP(attr_id)
    MAP(attr_assign)
    
    MAP(element_name)
    MAP(element)
    
    MAP(comment)
    
    MAP(indent)
    
    MAP(line)
    #undef MAP
    initialised = true;
}


struct Line {
    std::string content;
    smatch tree;
    std::vector<Line*> children;
    
    Line(const std::string& content_=""):
        content(content_){
    }
    
    ~Line(void){
        for(auto i=children.begin();i!=children.end();i++) delete *i;
    }
    
    std::string descendent_content(void){
        std::string text;
        for(auto child=children.begin();child!=children.end();child++){
            text += (*child)->content + "\n";
            text += (*child)->descendent_content();
        }
        return text;
    }
    
    void load(Stencil& stencil){
        for(auto child=children.begin();child!=children.end();child++){
            (*child)->make(stencil.root());
        }
    }
    
    void make(Xml::Node node){
        regex_match(content,tree,line);
        auto branch = tree.nested_results().begin();
        branch++; //Skip the indent
        const void* id = branch->regex_id();
        if(id==comment_) make_comment(node,*branch);
        else if(id==code_) make_code(node,*branch);
        else if(id==element_) make_element(node,*branch);
        else if(id==text_) make_text(node,*branch);
        else {
            for(auto child=children.begin();child!=children.end();child++){
                (*child)->make(node);
            }
        }
    }
    
    void make_comment(Xml::Node node,const smatch& tree){
        std::string comment;
        auto text = tree.nested_results().begin();
        if(text != tree.nested_results().end()) comment = text->str();
        std::string decendents = descendent_content();
        if(decendents.length()>0 and comment.length()>0) comment += "\n";
        comment += decendents;
        comment += " ";
        Xml::NodeAppendComment(node,comment);
    }
    
    void make_code(Xml::Node node,const smatch& tree){
        Xml::Node self = Xml::NodeAppend(node,"script");
        std::string lang = tree.str();
        //Add the type attribute
        Xml::NodeSetAttribute(self,"type","text/"+lang);
        //Add a comment token to escape "<![CDATA[" for HTML parsers
        if(lang=="r" or lang=="py") Xml::NodeAppendText(self,"#");
        //Concatenate the code
        // A starting newline is required to escape the commented "<![CDATA[" line
        std::string code = "\n" + descendent_content();
        //Add a comment token to escape "]]>". This needs to be added to the code string!
        if(lang=="r" or lang=="py") code += "#";
        Xml::NodeAppendCData(self,code);
    }
    
    void make_element(Xml::Node node,const smatch& tree){
        auto branch = tree.nested_results().begin();
        //First branch is a element_name or an attr
        auto element_name_or_attr = branch;
        //If its an element name get it, otherwise make it div
        std::string element_name;
        if(element_name_or_attr->regex_id()==element_name_){
            element_name = element_name_or_attr->str();
        } else {
            element_name = "div";
        }
        Xml::Node self = Xml::NodeAppend(node,element_name);
        for(auto branch = tree.nested_results().begin();branch!=tree.nested_results().end();branch++){
            const void* id = branch->regex_id();
            auto nested = branch->nested_results().begin();
            if(id==directive_include_) {
                auto identifier = nested;
                Xml::NodeSetAttribute(self,"data-include",identifier->str());
                if(branch->nested_results().size()>1){
                    auto selector = ++nested;
                    Xml::NodeSetAttribute(self,"data-select",selector->str());
                }
            }
            else if(id==directive_for_) {
                auto item = nested;
                auto expr = ++nested;
                Xml::NodeSetAttribute(self,"data-for",item->str()+":"+expr->str());
            }
            else if(id==directive_arg_ or id==directive_modifier_) {
                auto name = nested;
                auto arg = ++nested;
                Xml::NodeSetAttribute(self,"data-"+name->str(),arg->str());
            }
            else if(id==directive_noarg_){
                Xml::NodeSetAttribute(self,"data-"+branch->str(),"");
            }
            else if(id==attr_id_) {
                //Remove leading "#"
                Xml::NodeSetAttribute(self,"id",branch->str(0).erase(0,1));
            }
            else if(id==attr_class_){
                //Remove leading "."
                Xml::NodeAppendAttribute(self,"class",branch->str(0).erase(0,1));
            }
            else if(id==attr_assign_){
                auto nested = branch->nested_results().begin();
                auto name = nested;
                auto value = ++nested;
                //Remove leading and trailing quotes
                std::string string = value->str();
                string.erase(0,1);
                string.erase(string.length()-1,1);
                Xml::NodeSetAttribute(self,name->str(),string);
            }
            else if(id==text_) make_text(self,*branch);
        };
        
        for(auto child=children.begin();child!=children.end();child++){
            (*child)->make(self);
        }
    }
    
    void make_text(Xml::Node node,const smatch& tree){
        for(auto branch = tree.nested_results().begin();branch!=tree.nested_results().end();branch++){
            const void* id = branch->regex_id();
            if(id==chars_) Xml::NodeAppendText(node,branch->str());
            if(id==inlinee_) make_inline(node,*branch);
        }
    }
    
    void make_inline(Xml::Node node,const smatch& tree){
        auto branch = tree.nested_results().begin();
        std::string element_name = "span";
        auto expression = branch;
        if(tree.nested_results().size()==2){
            element_name = branch->str();
            expression = ++branch;
        }
        Xml::Node self = Xml::NodeAppend(node,element_name);
        Xml::NodeSetAttribute(self,"data-text",expression->str());
    }
    
    std::string print(std::string indent=""){
        std::string p = indent + " \"" + content + "\"\n";

        initialise();
        regex_match(content,tree,line);
        std::stringstream stream;
        print(tree,stream,indent+"  ");
        p += stream.str();
        
        for(auto i=children.begin();i!=children.end();i++){
            p += (*i)->print(indent+"  ");
        }
        return p;
    }
    
    void print(const smatch& node,std::ostream& stream,std::string indent=""){
        if(node.size()>0){
            auto regex_id = node.regex_id();
            std::string rule = rules[regex_id];
            stream<<indent<<rule<<": \""<<node.str(0)<<"\"\n";
        } else {
            stream<<indent<<"<empty>\n";
        }
        for(auto i=node.nested_results().begin();i!=node.nested_results().end();i++){
            print(*i,stream,indent+"  ");
        }
    }
};

Line parse(const std::string& stem) {
    //Create a root syntax tree node
    Line root;
    
    //Initialise structures for keeping track of parent-child relationships
    Line* parent = &root;
    Line* previous = &root;
    int current = 0;
    std::deque<std::pair<int,Line*>> levels;
    levels.push_back({-1,&root});
    
    //For each line...
    std::stringstream stream(stem);
    std::string string;
    while(std::getline(stream,string,'\n')){
        //Create a new Line
        Line* line = new Line(string);
        //Determine the parent-child relationships for this node based on its indentation
        int indent = string.find_first_not_of(" ");
        if(indent==(int)std::string::npos) indent = current;
        if(indent>levels.back().first){
            parent = previous;
            levels.push_back({indent,parent});
        } 
        else {
            while(indent<levels.back().first){
                levels.pop_back();
            }
            parent = levels.back().second;
        }
        parent->children.push_back(line);
        previous = line;
        current = indent;
    };
    
    return root;
}

//! A convienience function for converting Stem to HTML
std::string html(const std::string& stem) {
    Stencil stencil;
    stencil.stem(stem);
    return stencil.dump();
}

}

Stencil& Stencil::stem(const std::string& stem){
    Stem::parse(stem).load(*this);
    return *this;
}

}
