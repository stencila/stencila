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

#include <boost/algorithm/string.hpp>

#include <stencila/formats/xml.hpp>
namespace Xml = Stencila::Formats::Xml;
#include <stencila/reflect.hpp>
using namespace Stencila::Reflect;
#include <stencila/print.hpp>

namespace Stencila {
	
class Stencil : public Xml::Document {
	
private:

    class Walker : public Xml::Walker {
    
    private:
        Proxy& context_;
    
    public:
        Walker(Proxy& context):
            context_(context){
        }
        
        virtual bool for_each(Xml::Node& node) {
            BOOST_FOREACH(auto attribute, node.attributes()){
                std::string name = attribute.name();
                std::string value = attribute.value();
                if(name=="data-type"){
                    Proxy target = context_.get(value);
                    node.text().set(target.type().c_str());
                }
                else if(name=="data-repr"){
                    Proxy target = context_.get(value);
                    node.text().set(target.repr().c_str());
                }
                else if(name=="data-each"){
                    //Extract the var and iter tokens
                    std::vector<std::string> bits;
                    boost::split(bits,var_iter,boost::is_any_of(":"));
                    std::string var_name = bits[0];
                    std::string iter_name = bits[1];
                    //Get the iter from context
                    Proxy iter = context_.get(iter_name);
                    
                }
            }
            //Continue
            return true;
        }
    };

public:
    Stencil& render(Proxy context){
        Walker walker(context);
        this->traverse(walker);
        return *this;
    }
};
	
} 