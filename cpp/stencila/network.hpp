#pragma once

#include <iostream>
#include <map>

#include <boost/lexical_cast.hpp>

#define _WEBSOCKETPP_CPP11_STL_
#include <websocketpp/config/asio.hpp>
#include <websocketpp/server.hpp>

#include <stencila/host.hpp>
#include <stencila/component.hpp>

namespace Stencila {

using namespace websocketpp;
using namespace websocketpp::frame;
typedef server<websocketpp::config::asio> server;

class Server {
public:

	/**
	 * Construct a `Server`
	 */
	Server(void);

	/**
	 * Get the URL for this `Server`
	 */
	std::string url(void) const;

	/**
	 * Start the server
	 */
	void start(void);

	/**
	 * Stop the server
	 */
	void stop(void);

	/**
	 * Start server instance
	 */
	static std::string startup(void);

	/**
	 * Stop server instance
	 */
	static void shutdown(void);

private:

	/**
	 * Implementation of server
	 */
	server server_;
	
	/**
	 * Port number for the server
	 *
	 * The default port number, 7373, was chosen quasi-arbitarily from amongst the unassigned 
	 * port numbers at [IANA](http://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.txt)
	 * Seven and three also happen to be ["lucky numbers"](http://en.wikipedia.org/wiki/Lucky_number)!
	 */
	unsigned int port_ = 7373;

	/**
	 * An active session. Each session is linked to a component.
	 * This stores the component address that the session is related to.
	 */
	struct Session {
		std::string address;
	};
	typedef std::map<connection_hdl,Session,std::owner_less<connection_hdl>> Sessions;
	
	/**
	 * Mapping between a `connection_hdl` and a `Session`
	 */
	Sessions sessions_;

	/**
	 * Get the `Session` for a given `connection_hdl`
	 */
	Session& session_(connection_hdl connection);

	/**
	 * Convert a URL path to a Component address by 
	 * removing any leading or trailing forward slashes
	 */
	std::string address_(const std::string& path);

	/**
	 * Open a connection
	 * 
	 * @param connection Connection handle
	 */
	void open_(connection_hdl connection);
	
	/**
	 * Close a connection
	 * 
	 * @param connection Connection handle
	 */
	void close_(connection_hdl connection);

	/**
	 * Handle a HTTP request
	 * 
	 * @param connection Connection handle
	 */
	void http_(connection_hdl connection);

	/**
	 * Handle a websocket message
	 * 
	 * @param connection Connection handle
	 * @param message Message pointer
	 */
	void message_(connection_hdl connection, server::message_ptr message);

};

} // namespace Stencila
