#pragma once

#include <stencila/component.hpp>

namespace Stencila {

/**
 * Spread environments for Sheets
 * 
 * `Spreads` are to `Sheets` what `Contexts` are to `Stencils`. 
 * The spread is the execution environment for cell expressions.
 * Each cell in a sheet is represented within the attached spread by a 
 * variable.
 */
class Spread : public Component {
public:

	virtual ~Spread(void) {};

	/**
	 * Assign a expression to a variable name (and potentially an alias)
	 * 
	 * @param id ID of the cell
	 * @param expression Expression for the cell
	 * @param alias Alias name for the cell
	 */
	virtual std::string set(const std::string& id, const std::string& expression, const std::string& alias="") = 0;

	/**
	 * Get a text representation of a variable in the spread
	 * 
	 * @param name Could be a cell id e.g. EF5 or and alias e.g. price
	 */
	virtual std::string get(const std::string& name) = 0;

	/**
	 * Clear one or all cell values
	 * 
     * @param name Could be a cell id e.g. EF5 or and alias e.g. price
	 */
	virtual std::string clear(const std::string& name) = 0;

	/**
	 * List all the variables (ids and aliases) in the spread
	 * 
	 * @return  Comma separated list of names
	 */
	virtual std::string list(void) = 0;

	/**
	 * List the dependencies of a cell expression
     *
     * Parse a cell expression to obtain all it dependencies
     * This will include variables and functions, some of which
     * may not be in the sheet.
     * 
	 * @return  Comma separated list of names
	 */
	virtual std::string depends(const std::string& expression) = 0;

};

}
