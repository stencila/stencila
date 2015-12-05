#pragma once

#include <array>
#include <map>
#include <string>

#include <stencila/component.hpp>
#include <stencila/html.hpp>
//#include <stencila/spread.hpp>

namespace Stencila {

class Sheet : public Component {
public:

	Sheet(void);

	Sheet(const std::string& from);

	~Sheet(void);

	/**
	 * @name Attributes
	 *
	 * Methods for obtaining or setting attributes of the sheet.
	 *
	 * Methods implemented in `stencil-attrs.cpp`
	 * 
	 * @{
	 */
	
	/**
	 * Get the component type
	 */
	static Component::Type type(void);

	/**
	 * Get this sheets's title
	 */
	std::string title(void) const;

	/**
	 * Get this sheets's description
	 */
	std::string description(void) const;

	/**
	 * Get this sheets's keywords
	 */
	std::vector<std::string> keywords(void) const;

	/**
	 * Get this sheets's authors
	 */
	std::vector<std::string> authors(void) const;
	
	/**
	 * Get the list of spread that are compatible with this sheets.
	 *
	 * Context compatability will be determined by the expressions used in 
	 * sheets cells. Some expressions will be able to be used in multiple 
	 * spread languages.
	 */
	std::vector<std::string> contexts(void) const;

	/**
	 * Get this sheets's theme
	 *
	 * @param versioned Should the theme be returned with a version (if specified)?
	 */
	std::string theme(void) const;

	/**
	 * @}
	 */


	/**
	 * Initialise this sheet
	 * 
	 * @param  from A string indicating how the sheet is initialised
	 */
	Sheet& initialise(const std::string& from);

	/**
	 * Generate a HTML table for this sheet
	 */
	Html::Fragment html_table(unsigned int rows = 50, unsigned int cols = 20) const;

	/**
	 * Load this sheet from an input stream
	 * 
	 * @param  stream Input stream
	 */
	Sheet& load(std::istream& stream, const std::string& format = "tsv");

	/**
	 * Load this sheet from a string
	 * 
	 * @param  stream Input stream
	 */
	Sheet& load(const std::string& string, const std::string& format = "tsv");

	/**
	 * Dump this sheet to an output stream
	 * 
	 * @param  stream Output stream
	 */
	Sheet& dump(std::ostream& stream, const std::string& format = "tsv");

	/**
	 * Dump this sheet to a string
	 * 
	 * @param  format Format for dump
	 */
	std::string dump(const std::string& format = "tsv");

	/**
	 * Import this stencil content from a file
	 * 
	 * @param  path Filesystem path to file
	 */
	Sheet& import(const std::string& path);

	/**
	 * Export the stencil content to a file
	 * 
	 * @param  path Filesystem path to file
	 */
	Sheet& export_(const std::string& path);

	/**
	 * Read this sheet from a directory
	 * 
	 * @param  path Filesystem path to a directory. 
	 *              If an empty string then the sheet's current path is used.
	 */
	Sheet& read(const std::string& path="");

	/**
	 * Write this sheet to a directory
	 * 
	 * @param  path Filesystem path to a directory
	 *              If an empty string then the sheet's current path is used.
	 */
	Sheet& write(const std::string& path="");

	/**
	 * Generate a web page for a sheet
	 *
	 * @param  component  A pointer to a sheet
	 */
	static std::string page(const Component* component);

	/**
	 * Generate a web page for this sheet
	 */
	std::string page(void) const;

	/**
	 * Compile this sheet
	 *
	 * Export it as HTML to `index.html` in home directory
	 */
	Sheet& compile(void);

	/**
	 * A cell in the sheet; used to hold extra information other than
	 * it's content (e.g. equation)
	 */
	struct Cell {
		unsigned int row;
		unsigned int cell;
		std::string value;
		std::string expression;
		std::string alias;
	};

	/**
	 * Generate an identifier for a row
	 */
	static std::string identify_row(unsigned int row);

	/**
	 * Generate an identifier for column
	 */
	static std::string identify_col(unsigned int col);

	/**
	 * Generate an identifier for a cell based on its position
	 */
	static std::string identify(unsigned int row, unsigned int col);

	/**
	 * Parse a cell content into it's parts
	 */
	static std::array<std::string,3> parse(const std::string& content);

	

private:

	/**
	 * A map of cells having content
	 */
	std::map<std::string,Cell> cells_;

};

}
