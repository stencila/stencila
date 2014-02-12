#pragma once

#include <stencila/component.hpp>
#include <stencila/utilities/html.hpp>
using namespace Stencila::Utilities;

namespace Stencila {

class Stencil : public Component<Stencil>, public Xml::Document {
public:

    // Avoid ambiguities by defining which inherited method to use
    // when the base classes use the same name
    using Stencila::Component<Stencil>::path;

	typedef Xml::Attribute Attribute;
    typedef Xml::AttributeList AttributeList;
    typedef Xml::Node Node;

public:

    std::string type(void) const {
    	return "stencil";
    }

    /**
     * @name Contexts
     * @{
     */
    
private:

    std::vector<std::string> contexts_;

public:

    /**
     * Get the contexts that are supported by the stencil
     */
    const std::vector<std::string> contexts(void) const {
        return contexts_;
    }

    /**
     * Get the contexts that are supported by the stencil
     */
    std::vector<std::string> contexts(void) {
        return contexts_;
    }

    /**
     * Set the contexts that are supported by the stencil
     */
    Stencil& contexts(const std::vector<std::string>& values) {
        // Whitelist check of context values
        for(auto value : values){
            bool ok = false;
            for(auto context : {"py","r"}){
                if(value==context) ok = true;
            }
            if(not ok) STENCILA_THROW(Exception,"Context string not recognised: "+value)
        }
        contexts_ = values;
        return *this;
    }

    /**
     * @}
     */


    /**
     * @name Theme
     * @{
     */
    
private:

    std::string theme_ = "core/themes/default";

public:

    const std::string& theme(void) const {
        return theme_;
    }

    Stencil& theme(const std::string& theme) {
        theme_ = theme;
        return self();
    }

    /**
     * @}
     */


    /**
     * @name Sanitize
     * @{
     */
    
    static Xml::Whitelist whitelist;

    Stencil& sanitize(void) {
        Xml::Document::sanitize(whitelist);
    };

    /**
     * @}
     */


    /**
     * Read the stencil from a directory
     * 
     * @param  from Filesystem path to directory
     */
    Stencil& read(const std::string& from=""){
        
        Html::Document doc;
        doc.read(from);
        
        // Being a valid HTML5 document, doc already has a <head> <title> and <body>
        // so these do not have to be checked for
        Node head = doc.find("head");
        Node body = doc.find("body");

        // Title
        title(head.find("title").text());

        // Keywords
        if(Node elem = head.find("meta","name","keywords")){
            std::string content = elem.attr("content");
            std::vector<std::string> items;
            boost::split(items,content,boost::is_any_of(","));
            for(auto& keyword : items) boost::trim(keyword);
            keywords(items);
        }

        // Description
        if(Node elem = head.find("meta","name","description")){
            std::string content = elem.attr("content");
            description(content);
        }       

        // Contexts
        if(Node elem = body.find("ul","id","contexts")){
            std::vector<std::string> items;
            for(auto& item : elem.all("li")){
                items.push_back(item.text());
            }
            contexts(items);  
        }

        // Authors
        if(Node elem = body.find("address","id","authors")){
            std::vector<std::string> items;
            for(auto& item : elem.all("a[rel=\"author\"]")){
                items.push_back(item.text());
            }
            authors(items);  
        }

        // Content
        if(Node elem = body.find("main","id","content")){
            append_children(elem);
        }  

        // Sanitize before proceeding
        sanitize();     

        return self();
    }
    
    /**
     * Write the stencil to a directory
     * 
     * @param  to Filesystem path to directory
     */
    Stencil& write(const std::string& to=""){

        // Sanitize before writing
        sanitize();
        
        Html::Document doc;

        // Being a valid HTML5 document, doc already has a <head> <title> and <body>
        Node head = doc.find("head");
        Node body = doc.find("body");

        // Title to <title>
        head.find("title").text(title());

        // Keywords and description to <meta> tags
        head.append("meta",{
            {"name","keywords"},
            {"content",boost::algorithm::join(keywords(),", ")}
        });
        head.append("meta",{
            {"name","description"},
            {"content",description()}
        });

        // The following tags are added with a space as content so that they do not
        // get rendered as empty tags (e.g. <script... />). Whilst empty tags should be OK with XHTML
        // they can cause problems with some browsers.

        /**
         * <link rel="stylesheet" ...
         *
         * Links to CSS stylesheets are [placed in the head](http://developer.yahoo.com/performance/rules.html#css_top) 
         */
        std::string css = "http://stenci.la/" + theme() + "/base.min.css";
        head.append("link",{
            {"rel","stylesheet"},
            {"type","text/css"},
            {"href",css}
        }," ");

        /**
         * #contexts
         *
         * Added as a <ul> in body
         */
        
        Node contexts_elem = body.append("ul",{
            {"id","contexts"}
        }," ");
        for(auto context : contexts()){
            contexts_elem.append("li",{
                {"class",context}
            },context);
        }
		
        /**
         * #authors
         *
         * Use both <address> and <a rel="author" ...> as suggested at http://stackoverflow.com/a/7295013 .
         * The placement of <address> as a child of <body> should mean that this authors list applies to the whole document.
         * See:
            http://html5doctor.com/the-address-element/
            http://www.w3.org/TR/html5/sections.html#the-address-element
            http://stackoverflow.com/questions/7290504/which-html5-tag-should-i-use-to-mark-up-an-authors-name
         */
        auto authors_elem = body.append("address",{
            {"id","authors"}
        }," ");
        for(auto author : authors()){
            authors_elem.append("a",{
                {"rel","author"},
                {"href","#"}
            },author);
        }

        /**
         * #content
         *
         * Content is placed in a <div> rather than just using the <body> so that extra HTML elements can be added by the 
         * theme without affecting the stencil's content
         */
        auto content = body.append("main",{
            {"id","content"}
        });
        content.append(*this);

        /**
         * <script>
         *
         * Script elements are [placed at bottom of page](http://developer.yahoo.com/performance/rules.html#js_bottom)
         */
        std::string js = "http://stenci.la/" + theme() + "/base.min.js";
        body.append("script",{
            {"src",js}
        }," ");

        doc.write(to);
        
        return self();
    }


    /**
     * @name Embedding
     * @{
     */
    
private:

    static std::stack<Node> embed_parents_;

public:

    Stencil& embed(void) {
        unembed();
        embed_parents_.push(*this);
        return self();
    }

    Stencil& unembed(void) {
        embed_parents_ = std::stack<Node>();
        return self();
    }

    static Node element(const std::string& tag, const AttributeList& attributes, const std::string& text = ""){
        // Append to the current parent
        Node parent = embed_parents_.top();
        Node elem = parent.append(tag,attributes,text);
        return elem;
    }

    static Node element(const std::string& tag, const std::string& text){
        return element(tag,AttributeList(),text);
    }

    static Node element(const std::string& tag){
        return element(tag,AttributeList());
    }

    template<typename... Args>
    static Node element(const std::string& tag, const AttributeList& attributes, Args... args){
        Node started = start(tag,attributes);
        add(args...);
        Node finished = finish(tag);
        return started;
    }

    template<typename... Args>
    static Node element(const std::string& tag, Args... args){
        return element(tag,AttributeList(),args...);
    }

    static Node start(const std::string& tag, const AttributeList& attributes){
        Node elem = element(tag,attributes);
        embed_parents_.push(elem);
        return elem;
    }

    template<typename... Args>
    static void add(const std::string& text,Args... args){
        // Append a text node
        embed_parents_.top().append_text(text);
        add(args...);
    }

    template<typename... Args>
    static void add(Node& node,Args... args){
        // Append a node. This node must be moved from it's existing parent
        Node parent = embed_parents_.top();
        node.move(parent);
        add(args...);
    }

    template<typename... Args>
    static void add(void(*inner)(void),Args... args){
        // Execute the 
        inner();
        add(args...);
    }

    static void add(void){
    }

    static Node finish(const std::string& tag){
        Node elem = embed_parents_.top();
        embed_parents_.pop();
        return elem;
    }

    /**
     * @}
     */

};

Xml::Whitelist Stencil::whitelist = {
    {"p",{"class","data-text"}},
    {"div",{"class"}}
};

std::stack<Stencil::Node> Stencil::embed_parents_;

namespace Embed {

#define _(tag)\
    Html::Node tag(void){                                                                              return Stencil::element(#tag);                                 } \
    Html::Node tag(const std::string& text){                                                           return Stencil::element(#tag,text);                            } \
    Html::Node tag(const Stencil::AttributeList& attributes, const std::string& text=""){              return Stencil::element(#tag,attributes,text);                 } \
    Html::Node tag(void(*inner)(void)){                                                                return Stencil::element(#tag,Stencil::AttributeList(),inner);  } \
    template<typename... Args> Html::Node tag(const Stencil::AttributeList& attributes,Args... args){  return Stencil::element(#tag,attributes,args...);              } \
    template<typename... Args> Html::Node tag(Args... args){                                           return Stencil::element(#tag,args...);                         } \

_(section)
_(nav)
_(article)
_(aside)
_(address)
_(h1)
_(h2)
_(h3)
_(h4)
_(h5)
_(h6)
_(p)
_(hr)
_(pre)
_(blockquote)
_(ol)
_(ul)
_(li)
_(dl)
_(dt)
_(dd)
_(figure)
_(figcaption)
_(div)
_(a)
_(em)
_(strong)
_(small)
_(s)
_(cite)
_(q)
_(dfn)
_(abbr)
_(data)
_(time)
_(code)
_(var)
_(samp)
_(kbd)
_(sub)
_(sup)
_(i)
_(b)
_(u)
_(mark)
_(ruby)
_(rt)
_(rp)
_(bdi)
_(bdo)
_(span)
_(br)
_(wbr)
_(ins)
_(del)
_(table)
_(caption)
_(colgroup)
_(col)
_(tbody)
_(thead)
_(tfoot)
_(tr)
_(td)
_(th)

#undef _

}

}