#pragma once

#include <stencila/component.hpp>
#include <stencila/context.hpp>
#include <stencila/xml.hpp>

namespace Stencila {

class Theme : public Component {
public:

	/**
	 * Get the component type
	 */
	static Type type(void){
		return ThemeType;
	}

	/**
	 * Constructors
	 */
	Theme(void);
	Theme(const std::string& from);

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
	 * @name Metadata
	 *
	 * Methods for obtaining or setting metadata.
	 *
	 * Methods implemented in `theme-metadata.cpp`
	 * 
	 * @{
	 */

	/**
	 * Get this theme's title
	 */
	std::string title(void) const;

	/**
	 * Get this theme's description
	 */
	std::string description(void) const;

	/**
	 * Get this theme's keywords
	 */
	std::vector<std::string> keywords(void) const;

	/**
	 * Get this theme's authors
	 */
	std::vector<std::string> authors(void) const;
	
	/**
	 * Get this theme's theme
	 */
	std::string theme(void) const;

	/**
	 * @}
	 */

	
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
	 * Generate a web page for this theme
	 */
	std::string page(void) const;

	/**
	 * @}
	 */

private:

	std::string style_;
	
	std::string behaviour_;

	std::string title_;

	std::string description_;

	std::vector<std::string> keywords_;

	std::vector<std::string> authors_;

	std::string theme_ = "core/themes/themes/default";
};

}
