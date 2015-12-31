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
	 * Import a package
	 * 
	 * @param expression Expression in the host language
	 * @return     Type and text representation of cell value
	 */
	virtual std::string import(const std::string& package) = 0;

	/**
	 * Evaluate an expresion
	 * 
	 * @param expression Expression in the host language
	 * @return     Type and text representation of cell value
	 */
	virtual std::string evaluate(const std::string& expression) = 0;

	/**
	 * Assign a expression to a cell id and potentially a cell name
	 * 
	 * @param id ID of the cell
	 * @param expression Expression for the cell
	 * @param name Name for the cell
	 * @return     Type and text representation of cell value
	 */
	virtual std::string set(const std::string& id, const std::string& expression, const std::string& name="") = 0;

	/**
	 * Get a text representation of a variable in the spread
	 * 
	 * @param name Could be a cell id e.g. EF5 or and name e.g. price
	 * @return     Type and text representation of cell value
	 */
	virtual std::string get(const std::string& name) = 0;

	/**
	 * Clear one or all cells
	 * 
     * @param id ID of cell (if empty string clear all cells)
     * @param name Name of cell
	 */
	virtual std::string clear(const std::string& id = "", const std::string& name = "") = 0;

	/**
	 * List all the variables (ids and aliases) in the spread
	 * 
	 * @return  Comma separated list of names
	 */
	virtual std::string list(void) = 0;

	/**
	 * Collect a set of cells into an expression for the language
	 * 
	 * @param  cells List of cell ids
	 * @return       An expression in the host language
	 */
	virtual std::string collect(const std::vector<std::string>& cells) = 0;

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
