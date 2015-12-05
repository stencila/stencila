#pragma once

#include <stencila/component.hpp>

namespace Stencila {

class Spread : public Component {
public:

	virtual ~Spread(void) {};

	/**
	 * Execute code within the context
	 * 
	 * @param id ID of the cell
	 * @param expression Expression for the cell
	 * @param alias Alias name for the cell
	 */
	virtual std::string set(const std::string& id, const std::string& expression, const std::string& alias="") = 0;

	/**
	 * Get a text representation of a cell
	 * 
	 * @param  id ID of cell
	 */
	virtual std::string get(const std::string& id) = 0;

};

}
