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

//! @file rest-resource.hpp
//! @brief Defintion of a REST resource clas
//! @author Nokome Bentley

#pragma once

#include <string>

#include <stencila/http.hpp>
#include <stencila/json.hpp>

namespace Stencila {
namespace Rest {
    
template<class Derived=void>
class Resource {
public:

    Json::Document rest(const Http::Method& method, const Json::Document& json){
        switch(method.type){
            case Http::Method::POST: return static_cast<Derived*>(this)->post(json);
            case Http::Method::GET: return static_cast<Derived*>(this)->get();
            case Http::Method::PUT: return static_cast<Derived*>(this)->put(json);
            case Http::Method::PATCH: return static_cast<Derived*>(this)->patch(json);
            case Http::Method::DELETE: return static_cast<Derived*>(this)->del();
            default: STENCILA_THROW(Exception,"Unhandled HTTP method: "+method.string())
        }
    }
    
    std::string rest(const std::string& method, const std::string& json){
        return rest(Http::Method(method),Json::Document(json)).dump();
    }

    static Json::Document post(const Json::Document& json){
        Json::Document out;
        out.add("status","ok");
        return out;
    }
    
    Json::Document get(void){
        Json::Document out;
        out.add("status","ok");
        return out;
    }
    
    Json::Document put(const Json::Document& in){
        Json::Document out;
        out.add("status","ok");
        return out;
    }
    
    Json::Document patch(const Json::Document& in){
        Json::Document out;
        out.add("status","ok");
        return out;
    }
    
    Json::Document del(void){
        Json::Document out;
        out.add("status","ok");
        return out;
    }

};

}
}
