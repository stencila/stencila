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
	 * Initialise a sheet
	 * 
	 * @param  from A string indicating how the sheet is initialised
	 */
	//Sheet& initialise(const std::string& from);


	Sheet& read(std::istream& stream);

	/**
	 * Read the sheet from a directory
	 * 
	 * @param  path Filesystem path to a directory. 
	 *              If an empty string then the sheet's current path is used.
	 */
	Sheet& read(const std::string& path="");


	Sheet& write(std::ostream& stream);

	/**
	 * Write the sheet to a directory
	 * 
	 * @param  path Filesystem path to a directory
	 *              If an empty string then the sheet's current path is used.
	 */
	Sheet& write(const std::string& path="");


	struct Cell {
		unsigned int row;
		unsigned int cell;
		std::string content;
	};

	/**
	 * Generate an identifier for a cell based on its position
	 */
	static std::string identify(unsigned int row, unsigned int col);


	static std::array<std::string,3> parse(const std::string& content);

private:

	/**
	 * A map of cells having content
	 */
	std::map<std::string,Cell> cells_;

};

}
