#pragma once
#include <stencila/http-client.hpp>
#include <stencila/json.hpp>

namespace Stencila {

class Hub {
public:

	typedef Json::Document Document;

	Hub& signin(const std::string& username, const std::string& password);

	Hub& signin(const std::string& token);

	std::string username(void) const;

	Hub& signout(void);

	Document get(const std::string& url);

	Document post(const std::string& url);

private:

	Http::Client client_;

	static const std::string root_;

	std::string username_;

	std::string permit_;
};

extern Hub hub;

}
