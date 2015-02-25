#pragma once

#include <stencila/component.hpp>
#include <stencila/context.hpp>
#include <stencila/xml.hpp>

namespace Stencila {

class Theme : public Component {
public:

	Theme& make(void);

	/**
	 * @name Serving
	 *
	 * Methods for serving a theme over a nework.
	 * Overrides of `Component` methods as required.
	 *
	 * Methods implemented in `theme.cpp`
	 * 
	 * @{
	 */

	/**
	 * Serve this theme
	 */
	std::string serve(void);

	/**
	 * View this theme
	 */
	void view(void);

	/**
	 * Generate a web page for a theme
	 *
	 * @param  component  A pointer to a theme
	 */
	static std::string page(const Component* component);

	/**
	 * Generate a web page for this theme
	 */
	std::string page(void) const;

	/**
	 * Execute a call on a theme
	 *
	 * @param  component  A pointer to a theme
	 * @param  call       A `Call` object
	 */
	static std::string call(Component* component, const Call& call);

	/**
	 * Execute a call on this stencil
	 * 
	 * @param  call A `Call` object
	 */
	std::string call(const Call& call);

	/**
	 * @}
	 */

private:

	std::string style_;
	
	std::string behaviour_;
};

}
