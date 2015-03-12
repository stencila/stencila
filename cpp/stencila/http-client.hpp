#pragma once

#include <map>
#include <vector>
#include <string>

// Forward declarations
#include <boost/network/protocol/http/tags.hpp>
namespace boost {
	namespace network {
		namespace http {
			template <class Tag, unsigned version_major, unsigned version_minor> struct basic_client;
			typedef basic_client<tags::http_async_8bit_udp_resolve, 1, 1> client;
			template <class Tag> struct basic_request;
			typedef basic_request<tags::http_async_8bit_udp_resolve> request;
			template <class Tag> struct basic_response;
			typedef basic_response<tags::http_async_8bit_udp_resolve> response;
		}
	}
}

namespace Stencila {
namespace Http {

class Request {
public:

	/**
	 * Construct a request from a URL
	 */
	Request(const std::string& url);

	/**
	 * Construct a request from a URL, parameters and headers
	 */
	Request(
		const std::string& url,
		const std::map<std::string,std::string>& params,
		const std::map<std::string,std::string>& headers
	);

	/**
	 * Destruct a request
	 */
	~Request(void);

	Request& param(const std::string& name,const std::string& value);

	Request& header(const std::string& name,const std::string& value);

	Request& auth_basic(const std::string& username,const std::string& password);

	Request& body(const std::string& body);

private:

	/**
	 * Implementation
	 */
	typedef typename boost::network::http::request Implementation_;
	Implementation_* impl_;

	friend class Client;

};

class Response {
public:

	/**
	 * Get status code
	 */
	int status(void) const;

	/**
	 * Get values of headers with name
	 */
	std::vector<std::string> headers(const std::string& name) const;

	/**
	 * Get a cookie
	 */
	std::string cookie(const std::string& name) const;

	/**
	 * Get body
	 */
	std::string body(void) const;

	~Response(void);

private:

	/**
	 * Implementation
	 */
	typedef typename boost::network::http::response Implementation_;
	Implementation_* impl_;

	Response(const Implementation_& impl);

	// Can only be constructed by a client request
	friend class Client;
};

class Client {
public:

	/**
	 * Construct a `Client`
	 */
	Client(void);

	/**
	 * Destruct a `Client
	 */
	~Client(void);

	/**
	 * Make a GET request
	 */
	Response get(const Request& request);
	Response get(
		const std::string& url,
		const std::map<std::string,std::string>& params = {},
		const std::map<std::string,std::string>& headers = {}
	);

	/**
	 * Make a POST request
	 */
	Response post(const Request& request);
	Response post(
		const std::string& url,
		const std::map<std::string,std::string>& params = {},
		const std::map<std::string,std::string>& headers = {},
		const std::string& body = ""
	);

private:

	/**
	 * Implementation
	 */
	typedef typename boost::network::http::client Implementation_;
	Implementation_* impl_;

	/**
	 * Check a response. A private convenience method but
	 * seems to be necessary to wait for async fetch to finish
	 */
	void check_(typename boost::network::http::response& response);
};

Response get(
	const std::string& url,
	const std::map<std::string,std::string>& params = {},
	const std::map<std::string,std::string>& headers = {}
);

Response post(
	const std::string& url,
	const std::map<std::string,std::string>& params = {},
	const std::map<std::string,std::string>& headers = {},
	const std::string& body = ""
);

} // namespace Http
} // namespace Stencila
