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
#include <stencila/exception.hpp>

namespace Stencila {
namespace Http {

/*! 
 @namespace Http

 This namespace contains utility classes for handling the <a href="http://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol">Hypertext Transfer Protocol (HTTP)</a>.

*/

/*! 
 @class Method
*/
class Method {
public:
    enum Type {GET,HEAD,POST,PUT,DELETE,TRACE,OPTIONS,CONNECT,PATCH};
    
    Type type;
    
    Method(const Type method):
        type(method){
    }
    
    Method(const std::string& method) {
        if(method=="GET") type = GET;
        else if(method=="HEAD") type = HEAD;
        else if(method=="POST") type = POST;
        else if(method=="PUT") type = PUT;
        else if(method=="DELETE") type = DELETE;
        else if(method=="TRACE") type = TRACE;
        else if(method=="OPTIONS") type = OPTIONS;
        else if(method=="CONNECT") type = CONNECT;
        else if(method=="PATCH") type = PATCH;
        else {
            STENCILA_THROW(Exception,"Unknown HTTP method: "+type);
        }
    }
    
    //! @brief 
    //! @param other
    //! @return 
    bool operator==(const Method& other) const {
        return type==other.type;
    }

    //! @brief 
    //! @param other
    //! @return 
    bool operator!=(const Method& other) const {
        return type!=other.type;
    }
    
    std::string string(void) const {
        switch(type){
            case GET: return "GET";
            case HEAD: return "HEAD";
            case POST: return "POST";
            case PUT: return "PUT";
            case DELETE: return "DELETE";
            case TRACE: return "TRACE";
            case OPTIONS: return "OPTIONS";
            case CONNECT: return "CONNECT";
            case PATCH: return "PATCH";
        }
        return "";
    }
};

const Method Get(Method::GET);
const Method Head(Method::HEAD);
const Method Post(Method::POST);
const Method Put(Method::PUT);
const Method Delete(Method::DELETE);
const Method Trace(Method::TRACE);
const Method Options(Method::OPTIONS);
const Method Connect(Method::CONNECT);
const Method Patch(Method::PATCH);

//! @brief Get the Internet media type (MIME type) for a file extension
//!
//! See [Wikipedia](http://en.wikipedia.org/wiki/MIME_type) for more details
//! This only handles a limited number of file extensions
//! Python has a [mimetypes module](http://docs.python.org/2/library/mimetypes.html) with a mapping between extensions and MIME types
static std::string ContentType(const std::string& ext){
    if(ext==".txt") return "text/plain";
    if(ext==".css") return "text/css";
    if(ext==".html") return "text/html";
    
    if(ext==".png") return "image/png";
    if(ext==".svg") return "image/svg+xml";
    
    if(ext==".js") return "application/javascript";
    if(ext==".woff") return "application/font-wof";
    if(ext==".tff") return "application/font-ttf";
    
    return "";
}

}
}
