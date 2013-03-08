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

//! @file server.hpp
//! @brief Class for a RESTful HTTP server
//! @author Nokome Bentley

#pragma once

#include <string>
#include <iostream>
#include <fstream>
#include <sstream>
#include <algorithm>
#include <thread>

#include <boost/network/uri.hpp>
#include <boost/network/protocol/http/server.hpp>

#include <stencila/component.hpp>

namespace Stencila {
namespace Http {

using namespace boost::network;
using namespace boost::network::http;

#ifndef STENCILA_BROWSER_HOME
    #define STENCILA_BROWSER_HOME STENCILA_HOME "/browser"
#endif

class ServerHandler;
typedef boost::network::http::server<ServerHandler> ServerImpl;

//! @todo Log to a file
class ServerHandler {
public:

    typedef ServerImpl::request Request;
    typedef ServerImpl::response Response;
    typedef uri::uri Url;
    
    void operator() (const Request& request, Response& response) {
        try {
            std::string dest = destination(request);
            if(dest=="/") dest = "/file/index.html";
            Url url("http://localhost"+dest);
            
            std::string meth = method(request);
            
            std::vector<std::string> parts;
            std::string path = url.path();
            boost::split(parts,path,boost::is_any_of("/"));
            std::string type = parts.size()>1?parts[1]:"";
            
            if(meth=="GET" and type=="file") file(url,request,response);
            else if(meth=="POST" or meth=="GET" or meth=="PUT" or meth=="DELETE") rest(url,request,response);
            else error(405,"Method not supported",response);

            log(request,response);
        } catch (std::exception &e) {
            std::string what = std::string("Internal server error: ")+e.what();
            error(500,what,response);
        } catch (...) {
            error(500,"Internal server error",response);
        }
    }

private:

    void file(const Url& url, const Request& request, Response& response){
        // Attempt to open the file
        boost::filesystem::path filename(STENCILA_BROWSER_HOME);
        std::string path = url.path();
        //Remove "/file" from the start of the requested path
        path.erase(path.begin(),path.begin()+5);
        filename /= path;
        if(!boost::filesystem::exists(filename)) return error(404,"Not found: "+path,response);
        
        std::ifstream file(filename.string());
        if(!file.good()) return error(500,"Internal server error: file error",response);
        
        // Read file into a string
        //! @note There may be a [more efficient way to read a file into a string](http://stackoverflow.com/questions/2602013/read-whole-ascii-file-into-c-stdstring)
        std::string content((std::istreambuf_iterator<char>(file)),(std::istreambuf_iterator<char>()));
        
        // Get the MIME type
        std::string type = ContentType(filename.extension().string());
        
        // Setup the response
        response.status = ServerImpl::response::ok;
        response.headers.push_back({"Connection", "close"});
        response.headers.push_back({"Content-Type", type});
        response.content = content;
        response.headers.push_back({"Content-Length", boost::lexical_cast<std::string>(response.content.length())});
    }

    void rest(const Url& url, const Request& request, Response& response){
        // Call the Component REST method which does dispatching
        // based on HTTP method and URL
        std::string path = destination(request);
        std::string out = Component<>::rest(method(request),"http://localhost"+path,body(request));

        // Setup the response
        response.status = Response::ok;
        response.headers.push_back({"Connection", "close"});
        response.headers.push_back({"Content-Type", "application/json"});
        response.content = out;
        response.headers.push_back({"Content-Length", boost::lexical_cast<std::string>(response.content.length())});
    }
    
    void error(unsigned int code, const std::string& message, Response& response){
        response = ServerImpl::response::stock_reply(
            ServerImpl::response::status_type(code),
            "<!DOCTYPE html><html><head><title>Stencila Server Error</title></head><body><p>"+boost::lexical_cast<std::string>(code)+": "+message+"</p></body><html>"
        );
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
