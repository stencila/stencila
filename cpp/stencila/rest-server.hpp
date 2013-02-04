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

//! @file rest-server.hpp
//! @brief Class for a RESTful HTTP server
//! @author Nokome Bentley

#pragma once

#include <string>
#include <iostream>
#include <fstream>
#include <sstream>
#include <algorithm>
#include <thread>

#include <cpp-netlib/network/uri.hpp>
#include <cpp-netlib/network/uri/uri_io.hpp>
#include <cpp-netlib/network/protocol/http/server.hpp>

#include <stencila/http.hpp>
#include <stencila/rest-resource.hpp>
#include <stencila/registry.hpp>
#include <stencila/dataset.hpp>
#include <stencila/stencil.hpp>
#include <stencila/theme.hpp>

namespace Stencila {
namespace Rest {

using namespace boost::network;
using namespace boost::network::http;

using namespace Http;

#ifndef STENCILA_BROWSER_HOME
    #define STENCILA_BROWSER_HOME STENCILA_HOME "/browser"
#endif

class ServerHandler;
typedef boost::network::http::server<ServerHandler> ServerImpl;

//! @todo Log to a file
//! @todo Use a registry to POST new objects and PUT, PATCH, DELETE existing ones
class ServerHandler {
public:

    typedef ServerImpl::request Request;
    typedef ServerImpl::response Response;
    
    struct Url {
        
        std::string path;
        std::vector<std::string> bits;
        std::string query;
        
        Url(const std::string& url){
            uri::uri uri(url);
            path = uri.path();
            boost::split(bits,path,boost::is_any_of("/"));
            //Since the first part of the path is always "/", remove the first empty bit
            bits.erase(bits.begin());
            query = uri.query();
        }
        
        bool is_file() const {
            return path.find('.')!=std::string::npos;
        }
        
        std::string type(void) const {
            if(bits.size()>0) return bits[0];
            else return "";
        }
        
        std::string id(void) const {
            if(bits.size()>1) return bits[1];
            else return "";
        }
    };

    void operator() (const Request& request, Response& response) {
        try {
            std::string dest = destination(request);
            // If root return main HTML page
            if(dest=="/") dest = "/index.html";
            //Construct a URI object so that the path can be esracted without query or fragment components
            Url url("http://localhost"+dest);
            
            Method m = Method(method(request));
            switch(m.type){
                case Method::POST:
                    post(url,request,response);
                break;
                case Method::GET:
                    if(url.is_file()) serve(url.path,response);
                    else get(url,response);
                break;
                case Method::PUT:
                    put(url,request,response);
                break;
                case Method::DELETE:
                    del(url,response);
                break;
                default:
                    error(405,"Method not allowed",response);
                break;
            }
            log(request,response);
        } catch (std::exception &e) {
            std::string what = std::string("Internal server error: ")+e.what();
            error(500,what,response);
        } catch (...) {
            error(500,"Internal server error",response);
        }
    }

private:

    Json::Document json_get(const Request& request){
        std::string data = body(request);
        Json::Document json(data);
        return json;
    }
    
    void json_set(Json::Document& json, Response& response){
        response.status = Response::ok;
        response.headers.push_back({"Connection", "close"});
        response.headers.push_back({"Content-Type", "application/json"});
        response.content = json.dump();
        response.headers.push_back({"Content-Length", boost::lexical_cast<std::string>(response.content.length())});
    }
    
    void post(const Url& url, const Request& request, Response& response){
        Json::Document in = json_get(request);
        std::string type = url.type();
        std::string id = url.id();
        in.add("type",type);
        in.add("id",id);
        json_set(in,response);
    }
    
    void get(const Url& url, Response& response){
        Rest::Resource<> resource;
        Json::Document json = resource.get();
        json_set(json,response);
    }
    
    void put(const Url& url, const Request& request, Response& response){
    }
    
    void patch(const Url& url, const Request& request, Response& response){
    }
    
    void del(const Url& url, Response& response){
    }
    
    void serve(const std::string& path, Response& response){
        //Attempt to open the file
        boost::filesystem::path root(STENCILA_BROWSER_HOME);
        boost::filesystem::path filename = (root/path);
        std::ifstream file(filename.string());
        if(!file.good()) return error(404,"Not found: "+path,response);
        
        //Read file into a string
        //! @note There may be a [more efficient way to do this](http://stackoverflow.com/questions/2602013/read-whole-ascii-file-into-c-stdstring)
        std::string content((std::istreambuf_iterator<char>(file)),(std::istreambuf_iterator<char>()));
        
        //Get the MIME type
        std::string type = ContentType(filename.extension().string());
        
        //Setup the response
        response.status = ServerImpl::response::ok;
        response.headers.push_back({"Connection", "close"});
        response.headers.push_back({"Content-Type", type});
        response.content = content;
        response.headers.push_back({"Content-Length", boost::lexical_cast<std::string>(response.content.length())});
    }
    
public:
    
    //! @brief Output access log to std::cout using the Common Log Format
    //!
    //! For details on the Common Log Format see [here](http://en.wikipedia.org/wiki/Common_Log_Format) 
    //! and [here](http://www.w3.org/Daemon/User/Config/Logging.html#common-logfile-format)
    void log(const Request& request, Response& response){
        std::string hostname = source(request);//Remote hostname (or IP number if DNS hostname is not available)
        const char* logname = "-"; //The remote logname of the user
        const char* username = "-"; //The username as which the user has authenticated himself.
        const char* datetime = "[%d/%b/%Y:%H:%M:%S %z]";
        std::string meth = method(request);
        std::string dest = destination(request);
        //! @todo Reimplement this when new version of cpp-netlib available. Currently fails
        // std::string proto = protocol(request);
        const char* proto = "HTTP/1.0"; //The request HTTP protocol
        std::cout<<hostname<<" " 
                <<logname<<" "
                <<username<<" "
                <<datetime<<" "
                <<"\""<<meth<<" "<<dest<<" "<<proto<<"\" " //The request
                <<response.status<<" " //HTTP status code
                <<response.content.length()<<std::endl; //Size in bytes of object returned
    }
    
    void log(const std::string& error){
        std::cerr<<error<<std::endl;
    }
    
    void error(unsigned int code, const std::string& message, Response& response){
        response = ServerImpl::response::stock_reply(
            ServerImpl::response::status_type(code),
            "<!DOCTYPE html><html><head><title>Stencila::Rest::Server Error</title></head><body><p>"+boost::lexical_cast<std::string>(code)+": "+message+"</p></body><html>"
        );
    }
};

class Server {
private:
    ServerHandler handler_;
    ServerImpl server_;
    std::thread thread_;

public:

    Server(const std::string& address="localhost", const std::string& port="55555"):
        handler_(),
        server_(address,port,handler_){
    }
    
    void run(void) {
        server_.run();
    }
    
    void start(void) {
        thread_ = std::thread([&server_](){server_.run();});
    }
    
    void stop(void) {
        server_.stop();
        thread_.join();
    }
};

}
}
