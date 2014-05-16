#pragma once

#include <iostream>
#include <map>

#include <boost/lexical_cast.hpp>

#define _WEBSOCKETPP_CPP11_STL_
#include <websocketpp/config/asio.hpp>
#include <websocketpp/server.hpp>

#include <stencila/version.hpp>
#include <stencila/host.hpp>
#include <stencila/component.hpp>

namespace Stencila {
namespace Websocket {

using namespace websocketpp;
using namespace websocketpp::frame;
typedef server<websocketpp::config::asio> server;
using websocketpp::lib::placeholders::_1;
using websocketpp::lib::placeholders::_2;
using websocketpp::lib::bind;

class Server {
private:

	server server_;
	uint port_ = 9002;
	std::string name_;

	/**
	 * An active session. Each session is linked to a component.
	 * This stores the component address that the session is related to.
	 */
	struct Session {
		std::string address;
	};

	/**
	 * Mapping between a `connection_hdl` and a `Session`
	 */
	typedef std::map<connection_hdl,Session,std::owner_less<connection_hdl>> Sessions;
	Sessions sessions_;

	/**
	 * Get the `Session` for a given `connection_hdl`
	 */
	Session& session_(connection_hdl hdl) {
        auto i = sessions_.find(hdl);
        if(i==sessions_.end()) STENCILA_THROW(Exception,"No such session");
        return i->second;
    }

    /**
     * Open a connection
     */
    void open_(connection_hdl hdl) {
		server::connection_ptr connection = server_.get_con_from_hdl(hdl);
		std::string resource = connection->get_resource();
		std::string address = resource.substr(1);
		Session session;
		session.address = address;
        sessions_[hdl] = session;
    }
    
    /**
     * Close a connection
     */
    void close_(connection_hdl hdl) {
        Session& session = session_(hdl);
        sessions_.erase(hdl);
    }

    /**
     * Handle a HTTP request
     */
	void http_(connection_hdl hdl) {
		// Get the connection 
		server::connection_ptr connection = server_.get_con_from_hdl(hdl);
		// Get the remote address
		std::string remote = connection->get_remote_endpoint();
	    // Response variables
	    http::status_code::value status = http::status_code::ok;
	    std::string content;
		try {
		    // get_resource() returns "/" when there is no resource part in the URI
		    // (i.e. if the URI is just http://localhost/)
		    std::string path = connection->get_resource();
		    if(path=="/"){
				content = Component::home();
			} else {

			    // This server handles two types of requents for Components:
			    // (1) "Dynamic" requests where the component is loaded into
			    // memory (if not already) and (2) Static requests for component
			    // files
			    // Static requests are indicated by a "." anywhere in the url
			    bool dynamic = true;
			    auto found = path.find(".");
		  		if(found!=std::string::npos){
		  			dynamic = false;
		  		}

				if(dynamic){
					// Dynamic request
					// Remove the leading forward slash from the path to 
					// get the Component address
					std::string address = path.substr(1);
					content = Component::page(address);
				} else {
					// Static request
			        std::string filename = Component::resolve(path);
			        if(not filename.length()){
			        	// 404: not found
			        	status = http::status_code::not_found;
			        	content = "Not found: "+path;
			        } else {
			            std::ifstream file(filename);
			        	if(!file.good()){
			        		// 500 : internal server error
			        		status = http::status_code::internal_server_error;
			        		content = "Internal server error: file error";
			         	} else {
					        // Read file into content string
					        // There may be a [more efficient way to read a file into a string](
					        // http://stackoverflow.com/questions/2602013/read-whole-ascii-file-into-c-stdstring)
					        std::string file_content(
					        	(std::istreambuf_iterator<char>(file)),
					        	(std::istreambuf_iterator<char>())
					        );
			         		content = file_content;
			         		// Determine and set the "Content-Type" header
					        std::string content_type;
					        std::string extension = boost::filesystem::path(filename).extension().string();
					        if(extension==".txt") content_type = "text/plain";
					        else if(extension==".css") content_type = "text/css";
					        else if(extension==".html") content_type = "text/html";
					        else if(extension==".png") content_type = "image/png";
					        else if(extension==".jpg" | extension==".jpeg") content_type = "image/jpg";
					        else if(extension==".svg") content_type = "image/svg+xml";
					        else if(extension==".js") content_type = "application/javascript";
					        else if(extension==".woff") content_type = "application/font-wof";
					        else if(extension==".tff") content_type = "application/font-ttf";
					        connection->append_header("Content-Type",content_type);
			         	}
			        }
			    }
			}
		}
		catch(const std::exception& e){
			status = http::status_code::internal_server_error;
			content = "Internal server error: " + std::string(e.what());
		}
		catch(...){
			status = http::status_code::internal_server_error;
			content = "Internal server error: unknown exception";			
		}
	    // Replace the WebSocket++ "Server" header
	    connection->replace_header("Server",name_);
       	// Set status and content
	    connection->set_status(status);
	    connection->set_body(content);
	}

	/**
	 * Handle a websocket message
	 * 
	 * @param hdl Connection handle
	 * @param msg Message pointer
	 */
	void message_(connection_hdl hdl, server::message_ptr msg) {
		std::string response;
		try {
			Session session = session_(hdl);
			std::string request = msg->get_payload();
			std::cout<<request<<std::endl;
			response = Component::message(session.address,request);
			std::cout<<response<<std::endl;
		}
		// `Component::message()` should handle most exceptions and return a WAMP
		// ERROR message. If for some reason that does not happen, the following returns
		// a plain text error message...
		catch(const std::exception& e){
			response = "Internal server error : " + std::string(e.what());
		}
		catch(...){
			response = "Internal server error : unknown exception";			
		}
		server_.send(hdl,response,opcode::text);
	}

public:

	/**
	 * Construct a `Server`
	 */
	Server(void){
		// Set the name of this server (used in the HTPP Server header below)
		name_ = "Stencila ";
		name_ += version;
		// Initialise asynchronous IO
		server_.init_asio();
		// Set up handlers
		server_.set_open_handler(bind(&Server::open_,this,_1));
        server_.set_close_handler(bind(&Server::close_,this,_1));
        server_.set_http_handler(bind(&Server::http_,this,_1));
        server_.set_message_handler(bind(&Server::message_,this,_1,_2));
        // Turnoff logging
        server_.set_access_channels(log::alevel::none);
	}

	/**
	 * Get the URL for this `Server`
	 */
	std::string url(void) const {
		return "http://localhost:"+boost::lexical_cast<std::string>(port_);
	}

	/**
	 * Start the server
	 */
	void start(void){
		server_.listen(port_);
		server_.start_accept();
		server_.run();
	}

	/**
	 * Stop the server
	 */
	void stop(void){
		server_.stop();
	}

	static Server* instance_;
	static std::thread* thread_;
 
 	/**
 	 * Start server instance
 	 */
    static void startup(void) {
        if(not instance_){
        	instance_ = new Server();
        	thread_ = new std::thread([&instance_](){
        		instance_->start();
        	});
        }
    }

    static std::string ensure(void) {
    	startup();
    	return instance_->url();
    }
    
    /**
     * Stop server instance
     */
    static void shutdown(void) {
        if(instance_){
        	instance_->stop();
	        thread_->join();
	        delete instance_;
	        delete thread_;
	    }
    }
};

} // namespace Websocket
} // namespace Stencila
