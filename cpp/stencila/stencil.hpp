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
//! @author Nokome Bentley

#pragma once

#include <string>
#include <fstream>

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/date_time/posix_time/posix_time.hpp>
#include <boost/filesystem.hpp>

#include <stencila/xml.hpp>

namespace Stencila {

template<class Derived>
class Context {
public:

};

class EchoContext : public Context<EchoContext> {
public:

    //! @brief 
    //! @param name
    //! @param expression
    void set(const std::string& name, const std::string& expression){
    }

    //! @brief 
    //! @param code
    void script(const std::string& code){
    }

    //! @brief 
    //! @param expression
    //! @return 
    std::string text(const std::string& expression){
        return "text("+expression+")";
    }

    //! brief   
    //! @param expression
    //! @return 
    bool test(const std::string& expression){
        return false;
    }

    //! @brief 
    //! @param expression
    void subject(const std::string& expression){
    }

    //! @brief 
    //! @param expression
    //! @return 
    bool match(const std::string& expression){
        return false;
    }

    //! @brief 
    void enter(void){
    }
    
    //! @brief 
    //! @param expression
    void enter(const std::string& expression){
    }

    //! @brief 
    void exit(void){
    }

    //! @brief 
    //! @param item
    //! @param items
    //! @return 
    bool begin(const std::string& item,const std::string& items){
        return false;
    }

    //! @brief 
    //! @return 
    bool step(void){
        return false;
    }
};

class Stencil : public Xml::Document {
private:

    std::string id_;
    std::string uri_;

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
            Xml::NodeSetAttribute(node,"data-error",exc.what());
        }
        catch(...){
            Xml::NodeSetAttribute(node,"data-error","unknown error");
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
            Xml::NodeSetAttribute(node,"data-error",exc.what());
        }
        catch(...){
            Xml::NodeSetAttribute(node,"data-error","unknown error");
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
            Xml::NodeSetAttribute(node,"data-active","true");
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

    //! @brief 
    //! @param context
    //! @param identifier
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

    //! @brief 
    //! @return 
    Stencil(void){
    }
    
    //! @brief 
    //! @param content
    //! @return 
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

    //! @brief 
    //! @param html
    //! @return 
    Stencil& html(const std::string& html){
        load(html);
        return *this;
    }

    //! @brief 
    //! @param stem
    //! @return 
    Stencil& stem(const std::string& stem);
    static std::string stem_to_html(const std::string& stem);
    static std::string stem_to_string(const std::string& stem);

    //! @brief 
    //! @param path
    //! @return 
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

    //! @brief 
    //! @param id
    //! @return 
    Stencil& id(const std::string& id){
        return *this;
    }

    void identify(void){
        //Add an id if one does not yet exist
        if(id_.length()==0) id_ = boost::uuids::to_string(boost::uuids::random_generator()());
    }

    //! @brief 
    //! @return 
    std::string id(void) const {
        return id_;
    }

    //! @brief 
    //! @return 
    std::string uri(void) const {
        return uri_;
    }

    //! @brief 
    //! @param context
    //! @return 
    template<typename Context>
    Stencil& render(Context& context){
        render_element(*this,context);
        return *this;
    }
};

}
