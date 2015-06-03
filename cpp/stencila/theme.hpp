#pragma once

#include <stencila/component.hpp>
#include <stencila/context.hpp>
#include <stencila/xml.hpp>

namespace Stencila {

class Theme : public Component {
public:

	/**
	 * Initialise this theme
	 * 
	 * @param  from A string indicating how the theme is initialised
	 */
	Theme& initialise(const std::string& from);

	/**
	 * Get this theme's style
	 */
	std::string style(void) const;

	/**
	 * Set this theme's style
	 */
	Theme& style(const std::string& path);

	/**
	 * Get this theme's behaviour
	 */
	std::string behaviour(void) const;

	/**
	 * Set this theme's behaviour
	 */
	Theme& behaviour(const std::string& path);

	/**
	 * Read this theme from a directory
	 * 
	 * @param  path Filesystem path to a directory. 
	 *              If an empty string (`""`) then the current path, if any, is used.
	 */
	Theme& read(const std::string& directory="");
	
	/**
	 * Compile this theme
	 *
	 * Compile CSS (Cascading Style Sheets) and/or SCSS (Sass CSS) into
	 * minified CSS (`.min.css`) and Javascript into minified JS (`.min.js`)
	 */
	Theme& compile(void);

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
	Theme& view(void);

	/**
	 * Create a preview of this theme
	 */
	Theme& preview(const std::string& path);

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
