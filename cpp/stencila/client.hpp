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

//! @file client.hpp
//! @brief Class for a RESTful HTTP client
//! @author Nokome Bentley

#pragma once

#include <string>

#define BOOST_NETWORK_ENABLE_HTTPS
#include <cpp-netlib/network/protocol/http/client.hpp>

#include <stencila/http.hpp>
#include <stencila/json.hpp>

namespace Stencila {

using namespace boost::network;
using namespace boost::network::http;

class Client : public boost::network::http::client {
private:

    std::string address_;
    std::string port_;

    typedef boost::network::http::client::request Request;
    typedef boost::network::http::client::response Response;

    //! @brief Create a HTTP request with appropriate headers
    //!
    //! The following headers are set:
    //!   * Accept: Client accepts JSON content in the body of the response
    //!   * Accept-Encoding: Client accepts [gzip compressed content](http://en.wikipedia.org/wiki/HTTP_compression)
    //!   * Content-Type: Client is sending JSON content in the body of the request
    Request request_(const std::string& resource){
        std::string url = "https://"+address_+":"+port_+"/"+resource;
        Request request(url);
        request << header("Accept", "application/json")
                << header("Accept-Encoding", "gzip")
                << header("Content-Type", "application/json");
                << header("Connection", "close");
        return request;
    }
    
    Json::Document accept_(const Response& response){
        auto hdrs = headers(response);
        for(auto hdr : hdrs){
            std::cout<<hdr.first<<" "<<hdr.second<<"\n";
        }
        std::string json = response.body();
        return Json::Document(json);
    }

public:

    Client(const std::string& address="localhost", const std::string& port="55555"):
        address_(address),
        port_(port){
    }

    Json::Document get(const std::string& resource){
        return accept_(boost::network::http::client::get(request_(resource)));
    }

    Json::Document post(const std::string& resource,const std::string& data){
        return accept_(boost::network::http::client::post(request_(resource),data));
    }

    Json::Document put(const std::string& resource,const std::string& data){
        return accept_(boost::network::http::client::put(request_(resource),data));
    }

    Json::Document del(const std::string& resource){
        return accept_(boost::network::http::client::delete_(request_(resource)));
    }

};

}
