#pragma once

#include <array>
#include <map>
#include <string>

#include <stencila/component.hpp>
//#include <stencila/spread.hpp>

namespace Stencila {

class Sheet : public Component {
public:

	Sheet(void);

	Sheet(const std::string& from);

	~Sheet(void);

	/**
	 * Initialise this sheet
	 * 
	 * @param  from A string indicating how the sheet is initialised
	 */
	Sheet& initialise(const std::string& from);

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
		std::string content;
	};

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
