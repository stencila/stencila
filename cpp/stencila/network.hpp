#pragma once

#include <iostream>
#include <map>

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
	 * Get the hostname for this `Server`
	 */
	std::string hostname(void) const;

	/**
	 * Get the port for this `Server`
	 */
	std::string port(void) const;

	/**
	 * Get the origin (scheme+hostname+port) for this `Server`
	 */
	std::string origin(const std::string& scheme = "http") const;

	/**
	 * Get a URL for a scheme and path
	 */
	std::string url(const std::string& scheme, const std::string& path) const;

	/**
	 * Start the server
	 */
	void start(void);

	/**
	 * Send a websocket message  to a connection
	 * identfied by `id`
	 */
	void send(int id, const std::string& message);

	/**
	 * Stop the server
	 */
	void stop(void);

	/**
	 * Start server instance
	 */
	static const Server& startup(void);


	/**
	 * Get the current server instance
	 */
	static Server& instance(void);

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
	 * Hostname for the server
	 */
	std::string hostname_ = "localhost";
	
	/**
	 * Port number for the server
	 *
	 * The default port number, 7373, was chosen quasi-arbitarily from amongst the unassigned 
	 * port numbers at [IANA](http://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.txt)
	 * Seven and three also happen to be ["lucky numbers"](http://en.wikipedia.org/wiki/Lucky_number)!
	 */
	unsigned int port_ = 7373;

	/**
	 * Mapping between a `connection_hdl` and a connection id
	 */
	std::map<int, connection_hdl> connections_;

	/**
	 * Last connection id that was assigned
	 */
	int id_last_ = 0;

	/**
	 * Mapping between a connection id and a `connection_hdl`
	 */
	std::map<connection_hdl, int, std::owner_less<connection_hdl> > ids_;

	/**
	 * Access log file
	 */
	std::ofstream access_log_;

	/**
	 * Error log file
	 */
	std::ofstream error_log_;

	/**
	 * Keep track of the number of retires. See `start()` method.
	 */
	unsigned int restarts_;
	const static unsigned int max_restarts_ = 100;

	/**
	 * Restart the server after an otherwise unhandled exception
	 */
	void restart_(void);

	/**
	 * Get the path requested by a connection
	 */
	static std::string path_(server::connection_ptr connection);

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
	 * Receive a websocket message
	 * 
	 * @param connection Connection handle
	 * @param message Message pointer
	 */
	void receive_(connection_hdl connection, server::message_ptr message);

};

} // namespace Stencila
