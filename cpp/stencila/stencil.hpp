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

#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/date_time/posix_time/posix_time.hpp>

#include <stencila/registry.hpp>
#include <stencila/dataset.hpp>
#include <stencila/formats/xml.hpp>
namespace Xml = Stencila::Formats::Xml;

namespace Stencila {

template<class Derived>
class Context {
public:
};

class Stencil : public Xml::Document {
private:
    
    static Dataset registry_;

    std::string id_;
    
    template<typename Context>
    void element(Xml::Node node, Context& context){
        try {
            //Check for handled element tag names
            std::string tag = node.name();
            if(tag=="script") return script(node,context,node.text().as_string());
            //For each attribute in this node...
            auto attrs = node.attributes();
            for(auto attr=attrs.begin();attr!=attrs.end();attr++){
                //...get the name and value of attribute
                std::string name = attr->name();
                std::string value = attr->value();
                //...use the name of the attribute to dispatch to another method
                //   Note that return is used so that only the first Stencila "data-xxx" will be 
                //   considered and that directive will determin how/if children nodes are processed
                if(name=="data-text") return text(node,context,value);
                else if(name=="data-if") return if_(node,context,value);
                else if(name=="data-switch") return switch_(node,context,value);
                else if(name=="data-each") return each(node,context,value);
                else if(name=="data-with") return with(node,context,value);
                else if(name=="data-import") return import(node,context,value);
            }
            //If return not yet hit then process children of this element
            children(node,context);
        }
        catch(std::exception& exc){
            Xml::NodeSetAttribute(node,"data-error",exc.what());
        }
        catch(...){
            Xml::NodeSetAttribute(node,"data-error","unknown error");
        }
    }
    
    template<typename Context>
    void children(Xml::Node node, Context& context){
        for(Xml::Node child:node.children()){
            element(child,context);
        }
    }

    template<typename Context>
    void script(Xml::Node node, Context& context, const std::string& code){
         context.script(code);
    }

    template<typename Context>
    void text(Xml::Node node, Context& context, const std::string& expression){
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
    void with(Xml::Node node, Context& context, const std::string& expression){
        //Enter a new block in the context
        context.enter(expression);
        //Render all children of node within that new blockŔ
        children(node,context);
        //Exit the block
        context.exit();
    }

    template<typename Context>
    void if_(Xml::Node node, Context& context, const std::string& expression){
        //Test the expression
        bool result = context.test(expression);
        //If test passes, render all children
        if(result){
            children(node,context);
            Xml::NodeSetAttribute(node,"data-active","true");
        }
        //If test fails, remove the data-active attribute (if it exists)
        else {
            node.remove_attribute("data-active");
        }
    }

    template<typename Context>
    void switch_(Xml::Node node, Context& context, const std::string& expression){
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
            element(active,context);
        }
    }

    template<typename Context>
    void each(Xml::Node node, Context& context, const std::string& value){
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
                element(first,context);
            } else {
                //Create a copy of the first child node
                Xml::Node copy = node.append_copy(first);
                //Render the copy
                element(copy,context);
            }
            //Ask context to step
            more = context.step();
            count++;
        }
    }

    template<typename Context>
    void import(Xml::Node node, Context& context, const std::string& name){

        //Clear any existing children that have been included previously
        for(Xml::Node child : node.children()){
            if(Xml::NodeHasAttribute(child,"data-imported")){
                node.remove_child(child);
            }
        }

        // Get the HTML content of the included stencil
        //! @todo debug
        //auto row = Registry.row("SELECT * FROM \"stencils\" WHERE id==?",name);
        std::string content = "<span id=\"elem\" data-text=\"greeting\"></span>";//row[1];
        Xml::Document stencil(content);
        
        // Check to see if a subselection of modes is to be imported
        Xml::Attribute select = Xml::NodeGetAttribute(node,"data-select");
        if(select){
            std::string selector = select.value();
            Xml::Nodes imported = stencil.all(selector);
            for(auto i=imported.begin();i!=imported.end();i++){
                Xml::Node child = i->node();
                Xml::NodeSetAttribute(child,"data-imported");
                node.append_copy(child);
            }
        }
        //Otherwise import all children
        else {
            //Append the HTML content to the current node, setting the "data-imported" attribute
            for(Xml::Node child : stencil.children()){
                Xml::NodeSetAttribute(child,"data-imported");
                node.append_copy(child);
            }
        }

        //Apply child modifiers
        //! @todo deal with replace, before, after, prepend, append

        //Determine if any node parameters so that we don't create a new context block unecessarily
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
        children(node,context);
        
        //Exit the anonymous block if created
        if(params) context.exit();
    }

public:

    Stencil(void){
    }

    Stencil(const std::string& name){
        // See if a stencil with this name or id already exists.
        // If it does, load that stencil
        // If it does not, create a new stencil with name and a uid
        //! @todo implement
    }

    std::string id(void) const {
        return id_;
    }

    Stencil& save(void) {
        if(id_.length()==0) id_ = boost::uuids::to_string(boost::uuids::random_generator()());
        Registry.execute("INSERT INTO \"stencils\" VALUES(?,?)",id(),dump());
        return *this;
    }

    template<typename Context>
    Stencil& render(Context& context){
        element(*this,context);
        return *this;
    }
};

} 

