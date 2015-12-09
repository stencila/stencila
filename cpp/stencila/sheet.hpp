#pragma once

#include <array>
#include <map>
#include <string>
#include <vector>

#include <stencila/component.hpp>
#include <stencila/html.hpp>
#include <stencila/spread.hpp>

namespace Stencila {

class Sheet : public Component {
 public:
    /**
     * @name Construction and destruction.
     * 
     * @{
     */

    Sheet(void);

    explicit Sheet(const std::string& from);

    ~Sheet(void);

    /**
     * @}
     */


    /**
     * @name Attributes
     *
     * Methods for obtaining attributes of the sheet.
     * 
     * @{
     */

    /**
     * Get the component type for sheets
     */
    static Component::Type type(void);

    /**
     * Get this sheets's title
     *
     * Title is specified by using the title alias e.g
     *
     *    A1: title = Project X workbook
     *
     * Usually the title will be in cell A1 but it does not need
     * to be.
     */
    std::string title(void) const;

    /**
     * Get this sheets's description
     *
     * Description is specified by using the description alias e.g.
     *
     *     A2: description = Simple calculations for special project X
     */
    std::string description(void) const;

    /**
     * Get this sheets's keywords
     *
     * Keywords are specified in a comma separated cell 
     * with the keyword alias e.g.
     *
     *     A3: keywords = calculations, project X
     */
    std::vector<std::string> keywords(void) const;

    /**
     * Get this sheets's authors
     *
     * Authors are specified using a comma separated cell. Author identfiers
     * can be combinations of plain text, email addresses, usernames (`@` prefixed) or ORCIDs
     * e.g.
     *
     *     A4: authors = Peter Pan, Tinker Bell tinker@bell.name, @captainhook, 0000-0003-1608-7967 
     */
    std::vector<std::string> authors(void) const;

    /**
     * Get the list of spread types that are compatible with this sheet.
     *
     * Spreads provide the execution environment with with sheet calculations are performed.
     * Compatability will be determined by the expressions used in 
     * sheets cells. Some expressions will be able to be used in multiple 
     * spread languages.
     */
    std::vector<std::string> spreads(void) const;

    /**
     * Get this sheets's theme
     */
    std::string theme(void) const;

    /**
     * @}
     */


    /**
     * @name Input and output
     *
     * Initialising, loading and dumping, reading and
     * writing and conversion to/from other formats.
     * 
     * @{
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
    Sheet& read(const std::string& path = "");

    /**
     * Write this sheet to a directory
     * 
     * @param  path Filesystem path to a directory
     *              If an empty string then the sheet's current path is used.
     */
    Sheet& write(const std::string& path = "");

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
     * Export this sheet as HTML to `index.html` in home directory
     */
    Sheet& compile(void);

    /**
     * @}
     */


    /**
     * @name Serving
     */

    /**
     * Serve this stencil
     */
    std::string serve(void);

    /**
     * View this stencil
     */
    Sheet& view(void);

    /**
     * Respond to a web request to a sheet
     *
     * @param  component  A pointer to a sheet
     * @param  verb       HTML verb (a.k.a. method) e.g. POST
     * @param  method     Name of method requested
     * @param  body       Request body (usually JSON)
     */
    static std::string request(
        Component* component,
        const std::string& verb,
        const std::string& method,
        const std::string& body
    );

    /**
     * Respond to a web request to this sheet
     *
     * @param  verb       HTML verb (a.k.a. method) e.g. POST
     * @param  method     Name of method requested
     * @param  body       Request body (usually JSON)
     */
    std::string request(
        const std::string& verb,
        const std::string& method,
        const std::string& body
    );

    /**
     * @}
     */


    /**
     * @name Cells
     *
     * Cell contents, dependency analyses, updates etc
     */

    /**
     * A cell in the sheet.
     *
     * Rather than make this `struct Cell` too complex, functionality is
     * implemented as `Sheet` methods.
     */
    struct Cell {
        /**
         * Value of this cell
         *
         * Value may be empty if the cell has never been updated
         * or if it was updated and there was an error
         */
        std::string value;

        /**
         * Expression of this cell
         *
         * Expression may be empty if the cell is a constant.
         */
        std::string expression;

        /**
         * Alias for this cell
         *
         * Cells can have an alias. This is useful for writing more
         * meaningful and concise cell expression. For example, instead
         * of writing the expression for the area of a circle as,
         *
         *     A1: 3.14
         *     A2: 5
         *     A3: = A1*A2^2
         *
         * it could be written as,
         *
         *     A1: radius = 3
         *     A2: pi = 3.14
         *     A3: area = pi*radius^2
         *
         * Aliases must begin with a lower case letter but can include
         * both upper and lower case letters, digits and the underscore symbol.
         * The following are valid aliases:
         *
         *     a, a1, correction_factor_2
         *
         * but theses are not:
         *
         *     A, 1a, correction-factor-2  
         */
        std::string alias;
    };

    /**
     * Attach a spread to this stencil
     *
     * @param spread Spread for execution
     */
    Sheet& attach(std::shared_ptr<Spread> spread);

    /**
     * Detach the sheets's current spread
     */
    Sheet& detach(void);

    /**
     * Generate an identifier for a row
     *
     * Rows are identified by digits; this method merely
     * converts an `int` to a `string`
     */
    static std::string identify_row(unsigned int row);

    /**
     * Generate an identifier for column
     *
     * Columns are identified by combinations of uppercase
     * letters `A,B,C,...Z,AA,AB...`
     */
    static std::string identify_col(unsigned int col);

    /**
     * Generate an identifier for a cell based on its position
     *
     * Combines `identify_col()` and `identify_row()` into something
     * like `AT45` (but note that `row` is always the first argument!)
     */
    static std::string identify(unsigned int row, unsigned int col);

    /**
     * Parse a cell content into it's parts
     *
     * Parse the string content of a cell (e.g. from a `.tsv` file) into the
     * parts `value`, `expression`, `alias`. 
     * 
     * Tabs are converted to spaces. Spaces are insignificant before and after and alias
     * or expression. But they are significant at fron or end of a constant cell
     * 
     * If the content string does not match the
     * regex for `[<alias>] = <expression>` (ie optional alias), then it will be the
     * cell value (i.e. there is no invalid cell content except for a tab)
     *
     * @param content Cell content
     */
    static std::array<std::string, 3> parse(const std::string& content);

    /**
     * Update a cell's variable/s within the spread
     *
     * This method will set/update the cells corresponding
     * variable/s (id and potentially alias) within the spread environment
     *
     * @return The cell's current value
     */
    std::string update(const std::string& id, Cell& cell);

    /**
     * Update a cell with new content and its variable/s within the spread
     *
     * This method parses the new content and then calls `update(id, cell)`
     *
     * @return The cell's current value
     */
    std::string update(const std::string& id, const std::string& content);

    /**
     * Update all cells in this sheet
     *
     * This method might need to be called if for example a global variable
     * outside of the spread is altered
     */
    Sheet& update(void);

    /**
     * List the names of variables within the attached spread
     *
     * Variable names may include both ids (e.g. A1) and aliases (e.g. radius) 
     */
    std::vector<std::string> list(void);

    /**
     * Get the value of a variable within the attached spread
     * 
     * @param  name Name of variable (id or alias)
     */
    std::string value(const std::string& name);

    /**
     * Clear a cell
     *
     * After calling this method the cell will have no content and
     * no corresponding id (e.g. BD45) or alias (e.g. total) in the spread.
     * To remove an alias from a cell `update()` it content
     */
    Sheet& clear(const std::string& id);

    /**
     * Clear all cells
     */
    Sheet& clear(void);

 private:
    /**
     * A map of cells having content
     */
    std::map<std::string, Cell> cells_;

    /**
     * The current spread for this sheet
     */
    std::shared_ptr<Spread> spread_ = nullptr;
};

} // namespace Stencila
