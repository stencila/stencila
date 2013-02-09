/*
Copyright (c) 2012-2013 Stencila Ltd

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
//! @author Nokome Bentley

#pragma once

#include <string>
#include <fstream>

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>

#include <stencila/component.hpp>
#include <stencila/html.hpp>
#include <stencila/json.hpp>
#include <stencila/context.hpp>

namespace Stencila {

//! [Polyglot markup](http://www.w3.org/TR/html-polyglot/) is both HTML5 and XML. Some people call it XHTML5
//! There is a good summary of what XHTML5 requires [here](http://blog.whatwg.org/xhtml5-in-a-nutshell).
//! Note that this page should be served with the right MIME type i.e. "Content-Type: application/xhtml+xml" (although this is 
//! not supported by older versions of Microsoft IE (< 8.0?))
class Stencil : public Component<Stencil>, public Html::Document {
private:

    std::vector<std::string> keywords_;

public:

    static std::string type(void){
        return "stencil";
    };
    
    Stencil(void):
        Component<Stencil>(){
        from_scratch();
    }
    
    Stencil(const Id& id):
        Component<Stencil>(id){
        read();
    }

    //! @brief 
    //! @param content
    //! @return 
    Stencil(const std::string& content):
        Component<Stencil>(){
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
        if(type=="html") from_html(rest);
        else if(type=="stem") from_stem(rest);
        else if(type=="file") from_file(rest);
        else if(type=="id") from_id(rest);
        else STENCILA_THROW(Exception,"Unrecognised type: " + type)
    }
    
    //! @brief 
    //! @return 
    const std::vector<std::string> keywords(void) const {
        return keywords_;
    }
    
    //! @brief Create a stencil from scratch
    //!
    //! A XHTML5 document is created with a empty head and body
    //! Not that elements are added to the head element when the stencil is saved (see Stencil::dump)
    //!
    //! @return This stencil
    Stencil& from_scratch(void){
        prepend_doctype_html5();
        Node html = append("html",{{"xmlns","http://www.w3.org/1999/xhtml"}});
        append(html,"head");
        append(html,"body");
        return *this;
    }

    //! @brief 
    //!
    //! Certain elements within the head are parsed into stencil meta-data attributes e.g. meta name="keywords"
    //! Any other elements within the head will be ignored e.g. script, link
    //!
    //! @param html
    //! @return This stencil
    Stencil& from_html(const std::string& html){
        // Tidy HTML and load it into this stencil
        std::string html_tidy = tidy(html);
        load(html_tidy);
        
        //! @todo Extract metadata
        Node head = find("head");
        
        Node keywords = find(head,"meta","name","keywords");
        if(keywords){
            std::string content = Xml::Document::get(keywords,"content").value();
            boost::split(keywords_,content,boost::is_any_of(","));
            for(std::string& keyword : keywords_) boost::trim(keyword);
        }
        
        Node id = find(head,"meta","name","id");
        if(id){
            id_ = Xml::Document::get(id,"content").value();
        }
        
        // Now remove the extisting head and replace it with a new one
        remove(head);
        append(find("html"),"head");
        return *this;
    }

    //! @brief 
    //! @param stem
    //! @return 
    Stencil& from_stem(const std::string& stem);
    
    
    static std::string stem_print(const std::string& stem);

    //! @brief 
    //! @param path
    //! @return 
    Stencil& from_file(const std::string& path){
        std::ifstream file(path);
        std::stringstream buffer;
        buffer<<file.rdbuf();
        std::string ext = boost::filesystem::path(path).extension().string();
        if(ext==".html") {
            from_html(buffer.str());
        }
        else if(ext==".stem") {
            from_stem(buffer.str());
        } 
        else {
            STENCILA_THROW(Exception,"File extension not interpreted as a stencil:"+ext)
        }
        
        return *this;
    }

    //! @brief 
    //! @param id
    //! @return 
    Stencil& from_id(const std::string& id){
        return *this;
    }
    
    std::string body(void) {
        std::ostringstream out;
        for(Node child : find("body").children()) child.print(out,"",pugi::format_raw);
        return out.str();
    }
    
    Stencil& body(const std::string& html) {
        Html::Document html_doc(html);
        copy(find("body"),html_doc.find("body"));
        return *this;
    }
    
    //! @name Persistence methods
    //! @{
    
    Stencil& read(void){
        std::string dir = directory();
        if(boost::filesystem::exists(dir)){
            std::ifstream file(dir+"/index.html");
            std::string value((std::istreambuf_iterator<char>(file)),(std::istreambuf_iterator<char>()));
            body(value);
        }
        return *this;
    }
    
    Stencil& write(void) {
        std::string dir = directory();
        boost::filesystem::create_directories(dir);
        std::ofstream file(dir+"/index.html");
        file<<body();
        return *this;
    }
    
    //! @}
    
    
    //! @name REST interface methods
    //! @{
    
    std::string get(void){
        read();
        Json::Document out;
        out.add("body",body());
        return out.dump();
    }
    
    std::string put(const std::string& data){
        Json::Document json(data);
        if(json.has("body")) body(json.as<std::string>(json.get("body")));
        write();
        return "{}";
    }
    
    //! @}
    
    
    //! @name Rendering and display methods
    //! These methods provide alternative ways of rendering a stencil
    //! @{

    //! @brief Render a stencil into an HTML fragment
    //! @param context The context in which the stencil will be rendered
    //! @return The stencil
    template<typename Context>
    Stencil& render(Context& context){
        render_element(*this,context);
        return *this;
    }
    
private:

    //! @brief 
    //! @param node
    //! @param context
    //! @return 
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
        //! @brief 
        //! @param exc
        //! @return 
        catch(std::exception& exc){
            Xml::Document::set(node,"data-error",exc.what());
        }
        catch(...){
            Xml::Document::set(node,"data-error","unknown error");
        }
    }
    
    //! @brief 
    //! @param context
    template<typename Context>
    void render_children(Xml::Node node, Context& context){
        for(Xml::Node child:node.children()){
            render_element(child,context);
        }
    }

    //! @brief 
    //! @param context
    //! @param code
    template<typename Context>
    void render_script(Xml::Node node, Context& context, const std::string& code){
         context.script(code);
    }

    //! @brief 
    //! @param context
    //! @param expression
    template<typename Context>
    void render_text(Xml::Node node, Context& context, const std::string& expression){
        try {
            std::string text = context.text(expression);
            node.text().set(text.c_str());
        }
        catch(std::exception& exc){
            Xml::Document::set(node,"data-error",exc.what());
        }
        catch(...){
            Xml::Document::set(node,"data-error","unknown error");
        }
    }

    //! @brief 
    //! @param context
    //! @param expression    
    template<typename Context>
    void render_with(Xml::Node node, Context& context, const std::string& expression){
        //Enter a new block in the context
        context.enter(expression);
        //Render all children of node within that new blockŔ
        render_children(node,context);
        //Exit the block
        context.exit();
    }

    //! @brief 
    //! @param context
    //! @param expression
    template<typename Context>
    void render_if(Xml::Node node, Context& context, const std::string& expression){
        //Test the expression
        bool result = context.test(expression);
        //If test passes, render all children
        if(result){
            render_children(node,context);
            Xml::Document::set(node,"data-active","true");
        }
        //If test fails, remove the data-active attribute (if it exists)
        else {
            node.remove_attribute("data-active");
        }
    }

    //! @brief 
    //! @param context
    //! @param expression
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
            Xml::Attribute when = Xml::Document::get(child,"data-value");
            if(when){
                bool equal = context.match(when.value());
                if(equal){
                    active = child;
                    break;
                }
            } else if(Xml::Document::get(child,"data-default")){
                active = child;
            }
        }
        if(active){
            //Set as active
            Xml::Document::set(active,"data-active","true");
            //Render it
            render_element(active,context);
        }
    }

    //! @brief 
    //! @param context
    //! @param value
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
        Xml::Node first = node.find_child(Xml::Document::is_element);
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

    //! @brief 
    //! @param context
    //! @param identifier
    template<typename Context>
    void render_include(Xml::Node node, Context& context, const std::string& identifier){

        //Remove any existing children that have been included previously
        for(Xml::Node child : node.children()){
            if(Xml::Document::has(child,"data-included")){
                node.remove_child(child);
            }
        }

        // Get the included stencil
        Stencil source(identifier);
        Xml::Document sink;
        // Check to see if a subselection of modes is to be included
        Xml::Attribute select = Xml::Document::get(node,"data-select");
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
                Xml::Attribute attr = Xml::Document::get(child,attr_name);
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
            Xml::Document::set(child,"data-included","true");
            node.append_copy(child);
        }

        //Create a new context with parameters
        //Determine if there are any node parameters so that we don't create a new context block unecessarily
        bool params = Xml::Document::has(node,"data-param");
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

    //! @brief Dump the stencil into a string
    //!
    //! Serialise meta-data into head
    //! @return String representation of stencil
    std::string dump(void){
        Node head = find("head");
        append(head,"title","Stencil "+id());
        
        append(head,"meta",{
            {"charset","utf-8"}
        },"");
        append(head,"meta",{
            {"name","generator"},
            {"content","Stencila"}
        });
        append(head,"meta",{
            {"name","id"},
            {"content",id()}
        });
        append(head,"script",{
            {"type","text/javascript"},
            {"src","stencila-boot.js"},
        },"");
        return Html::Document::dump();
    }
};

}
