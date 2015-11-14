#pragma once
#include <stencila/http-client.hpp>
#include <stencila/json.hpp>

namespace Stencila {

class Hub {
public:

	/**
	 * Construct a hub instance
	 */
	Hub(void);

	/**
	 * Sign in using username and password.
	 *
	 * You should never use this method with a hardcoded password e.g.
	 *
	 *   hub.signin("my-username","my-password-which-is-easily-discoverable");
	 *
	 * since that would leak your password. Rather, this method is intended
	 * to be called by Stencila language packages (e.g R, Python) to provide
	 * a means to signin. 
	 * 
	 * @param  username Username for https://stenci.la
	 * @param  password Password for https://stenci.la
	 */
	Hub& signin(const std::string& username, const std::string& password);

	/**
	 * Sign in using a user token.
	 *
	 * User tokens provide an alternative to using your username/password
	 * pair for signing in. Tokens are unique strings of characters which identify
	 * you but have an expiry time and can be easily revoked. Tokens need to
	 * be kept secure so you should never use this method with a hardcoded token e.g.
	 *
	 * 	 hub.signin("01gk9dO72VEt9iBOaGdims9ADecQyScaKdnaYjnDucH6bgSAvSvYZ+JQ4DkDiNWziBwJx6yXTH+fzKL6GKUA==");
	 *
	 * Instead, you should use the `signin()` method (i.e. the version
	 * which takes no arguments) which tries to obtain a token from the 
	 * environment variable `STENCILA_HUB_TOKEN`
	 * 
	 * @param  token Token string obtained from https://stenci.la
	 */
	Hub& signin(const std::string& token);

	/**
	 * Sign in using user token defined in the environment variable `STENCILA_HUB_TOKEN`
	 *
	 * This is the preferred method for signing into the Stencila hub 
	 * using a remote machine not being used by a real person.  
	 */
	Hub& signin(void);

	/**
	 * Get the username of the user that is currently signed in
	 *
	 * @return Username string
	 */
	std::string username(void) const;

	/**
	 * Signout
	 */
	Hub& signout(void);

	/**
	 * Make a HTTP request to the hub
	 * 
	 * @param  path Path to the resource
	 * @return A JSON document
	 */
	Json::Document request(Http::Method method, const std::string& path);

	/**
	 * Get something from the hub
	 * 
	 * @param  path Path to the resource
	 * @return A JSON document
	 */
	Json::Document get(const std::string& path);

	/**
	 * Post something to the hub
	 * 
	 * @param  path Path to the resource
	 * @return A JSON document
	 */
	Json::Document post(const std::string& path);

	/**
	 * Delete something from the hub
	 * 
	 * @param  path Path to the resource
	 * @return A JSON document
	 */
	Json::Document delete_(const std::string& path);

	/**
	 * Clone a component repository from the hub
	 * 
	 * @param  address Address of component to be cloned
	 * @return Path to newly cloned component
	 */
	std::string clone(const std::string& address);

	/**
	 * Fork a component repository from the hub
	 * 
	 * @param  from Address of component to be forked
	 * @param  to Address to fork to
	 * @return Path to new fork
	 */
	std::string fork(const std::string& from, const std::string& to);

private:

	Http::Client client_;

	std::string root_;

	std::string username_;

	std::string permit_;
};

extern Hub hub;

}
