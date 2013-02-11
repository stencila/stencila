/*
Copyright (c) 2013 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file http.hpp
//! @brief Classes and functions for working with HTTP
//! @author Nokome Bentley

#pragma once

#include <string>

#include <boost/algorithm/string.hpp>
#include <boost/network/uri.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Http {

/*! 
 @namespace Http

 This namespace contains utility classes for handling the <a href="http://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol">Hypertext Transfer Protocol (HTTP)</a>.

*/



//! http://cpp-netlib.org/0.9.4/in_depth/uri.html
class Uri {
private:

    std::vector<std::string> path_;
    std::vector<std::vector<std::string>> fields_;
    std::string fragment_;

public:
    
    Uri(const std::string& url){
        // Using cpp-netlib uri class for parsing, although
        // the functionality used here should be fairly straightforward to implement
        boost::network::uri::uri uri(url);
        // Split the path up
        // Since the first part of the path is always "/" the first element
        // of bits is always empty so erase it
        std::string path = uri.path();
        boost::split(path_,path,boost::is_any_of("/"));
        path_.erase(path_.begin());
        // Split the query into name=value pairs
        std::string query = uri.query();
        std::vector<std::string> pairs;
        boost::split(pairs,query,boost::is_any_of("&"));
        for(std::string pair : pairs){
            std::vector<std::string> field;
            boost::split(field,pair,boost::is_any_of("="));
            fields_.push_back(field);
        }
        // Assign fragment
        fragment_ = uri.fragment();
    }
    
    std::string path(unsigned int index,const std::string& defaul="") const {
        return path_.size()>index?path_[index]:defaul;
    }
    
     std::vector<std::vector<std::string>> fields(void) const {
        return fields_;
    }
    
    std::string fragment(void) const {
        return fragment_;
    }
    
};

/*! 
 @class Method
*/
class Method : public std::string {
public:
    
    Method(const std::string& method) {
        if(method!="GET" and method!="HEAD" and method!="POST"
            and method!="PUT" and method!="DELETE" and method!="TRACE"
            and method!="OPTIONS" and method!="CONNECT" and method!="PATCH"){
            STENCILA_THROW(Exception,"unknown HTTP method: "+method);
        }
        else assign(method);
    }
};

const Method Get("GET");
const Method Head("HEAD");
const Method Post("POST");
const Method Put("PUT");
const Method Delete("DELETE");
const Method Trace("TRACE");
const Method Options("OPTIONS");
const Method Connect("CONNECT");
const Method Patch("PATCH");

//! @brief Get the Internet media type (MIME type) for a file extension
//!
//! See [Wikipedia](http://en.wikipedia.org/wiki/MIME_type) for more details
//! This only handles a limited number of file extensions
//! Python has a [mimetypes module](http://docs.python.org/2/library/mimetypes.html) with a mapping between extensions and MIME types
class ContentType : public std::string {
public:
    ContentType(const std::string& ext){
        if(ext==".txt") assign("text/plain");
        if(ext==".css") assign("text/css");
        if(ext==".html") assign("text/html");
        
        if(ext==".png") assign("image/png");
        if(ext==".svg") assign("image/svg+xml");
        
        if(ext==".js") assign("application/javascript");
        if(ext==".woff") assign("application/font-wof");
        if(ext==".tff") assign("application/font-ttf");
    }
};

}
}
