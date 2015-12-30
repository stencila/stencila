#pragma once

#include <array>
#include <map>
#include <string>
#include <vector>

#include <boost/regex.hpp>
#include <boost/graph/adjacency_list.hpp>

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
     * Get a meta attribute of this sheet
     * 
     * @param  what Name of attribute
     */
    std::string meta(const std::string& what) const;

    /**
     * Get this sheets's title
     *
     * Title is specified by using the title name e.g
     *
     *    A1: title = "Project X workbook"
     *
     * Usually the title will be in cell A1 but it does not need
     * to be.
     */
    std::string title(void) const;

    /**
     * Get this sheets's description
     *
     * Description is specified by using the description name e.g.
     *
     *     A2: description = "Simple calculations for special project X"
     */
    std::string description(void) const;

    /**
     * Get this sheets's keywords
     *
     * Keywords are specified in a comma separated cell 
     * with the keyword name e.g.
     *
     *     A3: keywords = "calculations, project X"
     */
    std::vector<std::string> keywords(void) const;

    /**
     * Get this sheets's authors
     *
     * Authors are specified using a comma separated cell. Author identfiers
     * can be combinations of plain text, email addresses, usernames (`@` prefixed) or ORCIDs
     * e.g.
     *
     *     A4: authors = "Peter Pan, Tinker Bell tinker@bell.name, @captainhook, 0000-0003-1608-7967"
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
    Html::Fragment html_table(unsigned int rows = 0, unsigned int cols = 0) const;

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
     *
     * Methods for serving a sheet over a network.
     * Overides of `Component` methods as required.
     *
     * @{
     */

    /**
     * Serve this sheet
     */
    std::string serve(void);

    /**
     * View this sheet
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
         * Directive for this cell
         *
         * Directives are "special" cells for doing things with a sheet.
         * Most cells have a empty string for their directive and are treated
         * as expressions in the host language
         */
        std::string directive;

        /**
         * Expression of this cell
         *
         * All cells have an expression the expression may be a literal (e.g. "hello", 42, 3.14) or
         * a formula (e.g. 2*sum(A1:B5)).
         *
         * Making all cell expressions means that it is slightly more burdensome for a user to write a
         * cell with a text value because they must make it a string expression (i.e `"Hello world"` instead of just `Hello world`).
         * But it makes a sheet a lot closer, and the user more familiar with, programming in the host language.
         */
        std::string expression;

        /**
         * Name for this cell
         *
         * Cells can have an name. This is useful for writing more
         * meaningful and concise cell expression. For example, instead
         * of writing the expression for the area of a circle as,
         *
         *     A1: 5
         *     A2: 3.14
         *     A3: A1*A2^2
         *
         * it could be written as,
         *
         *     A1: radius = 3
         *     A2: pi = 3.14
         *     A3: area = pi*radius^2
         *
         * Names must begin with a lower case letter but can include
         * both upper and lower case letters, digits and the underscore symbol.
         * The following are valid names:
         *
         *     a, a1, correction_factor_2
         *
         * but theses are not:
         *
         *     A, 1a, correction-factor-2  
         */
        std::string name;

        /**
         * Variables that this cell depends upon.
         * Used to determine dependency graph
         */
        std::vector<std::string> depends;

        /**
         * Type of this cell
         *
         * The type of object (e.g. integer, instance of a class, image) in the cell.
         * Used for determining the display and interactions in the user interface.
         */
        std::string type;

        /**
         * Value of this cell
         *
         * Value when expression is evaluated. A string representation of the underlying object.
         * Value may be empty if the cell has never been updated
         * or if it was updated and there was an error
         */
        std::string value;
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
     * converts an `int` to a `string`. The inverse of `index_row`
     */
    static std::string identify_row(unsigned int row);

    /**
     * Generate an identifier for column
     *
     * Columns are identified by combinations of uppercase
     * letters `A,B,C,...Z,AA,AB...` The inverse of `index_col`
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
     * Regular expression used for identifiying and parsing cell IDs
     */
    static boost::regex id_regex;

    /**
     * Is a string a valid cell ID?
     */
    static bool is_id(const std::string& id);

    /**
     * Generate a row index from a row identifier.
     *
     * The inverse of `indentify_row`
     */
    static unsigned int index_row(const std::string& row);

    /**
     * Generate a column index from a column identifier.
     *
     * The inverse of `indentify_col`
     */
    static unsigned int index_col(const std::string& col);

    /**
     * Generate the row and column index from a cell identifier
     */
    static std::array<unsigned int, 2> index(const std::string& id);

    /**
     * Create a list of cell IDs that interpolate between the
     * upper left and bottom right corners.
     */
    static std::vector<std::string> interpolate(
        const std::string& col1, const std::string& row1,
        const std::string& col2, const std::string& row2
    );

    /**
     * Get the extent of the sheet (maximum row and colum indices having cells)
     *
     * @return  Row, column 0-based indices
     */
    std::array<unsigned int, 2> extent(void) const;

    /**
     * Parse a cell content into it's parts
     *
     * Parse the string content of a cell (e.g. from a `.tsv` file) into the
     * parts `expression` and `name`. 
     * 
     * Tabs are converted to spaces. Spaces are insignificant before and after an
     * expression but are significant within an expression (i.e. expressions are trimmed)
     *
     * @param content Cell content
     */
    static std::array<std::string, 3> parse(const std::string& content);

    /**
     * Translate a sheet expression into an expression for the host language
     * of the attached spread
     * 
     * At present the only expression parts required to be translated are
     *   cell sequence `:` e.g. A1:A3 -> c(A1,A2,A3)
     *   cell union    '&' e.g. A1&A2:A3&A4 -> c(A1,A2,A3,A4)
     *
     * Regexes are used to ensure this translation is only done for operators
     * applied to cell ids. 
     * 
     * Note that '&' is used as the cell union operator although 
     * in Excel and Libre Office a comma is used. Commas are more easily confused 
     * with function argument delimiters. The ampersand, is used in some languages
     * for set union (e.g. Python) although in R '&' is the logical "and" operator. 
     */
    std::string translate(const std::string& expression);

    /**
     * Evaluate a sheet expression within the attached spread
     */
    std::array<std::string, 2> evaluate(const std::string& expression);

    /**
     * Get the source (expression and, optionally, name) for a cell
     * 
     * @param  id ID of the cell
     */
    std::string source(const std::string& id);

    /**
     * Set the source of a cell
     *
     * Note that this method does not do any cell calculations
     * That must be done using the `update()` methods (which take into account cell inter-dependencies)
     * 
     * @param  id      ID of the cell
     * @param  source New source
     */
    Sheet& source(const std::string& id, const std::string& source);

    /**
     * Update cells with new source
     *
     * This method parses the new source and will then set/update the cells corresponding
     * variable/s (both id and optional name) within the spread environment. Because of
     * interdependencies between cells this method is designed to take batches of cell updates,
     * analyse the dependency graph and then execute each cell expression.
     *
     * @param cells Map of cell IDs and their sources
     * @return List of IDs of the cells that have changed (including updated cells and their successors)
     */
    std::map<std::string, std::array<std::string, 2>> update(const std::map<std::string,std::string>& cells);

    /**
     * Update a single cell with new source
     * 
     * @param  id      ID of the cell
     * @param  source Cell source
     * @return         New value of the cell
     */
    std::map<std::string, std::array<std::string, 2>> update(const std::string& id, const std::string& source);

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
     * Variable names may include both ids (e.g. A1) and names (e.g. radius) 
     */
    std::vector<std::string> list(void);

    /**
     * Get the content (type+value) of a variable within the attached spread
     * 
     * @param  name Name of variable (id or name)
     */
    std::string content(const std::string& name);

    /**
     * Get a list of the cells that a cell depends upon (i.e. it's direct predecessors)
     *
     * @param id ID of the cell
     */
    std::vector<std::string> depends(const std::string& id);

    /**
     * Get the topological sort order for the cells in this sheet
     */
    std::vector<std::string> order(void);

    /**
     * Generate a Graphviz `dot` file of the dependency graph for this sheet
     *
     * @param path  File system path for the `.dot` file
     * @param image Should the `dot` program be called to prdocue a PNG?
     */
    void graphviz(const std::string& path, bool image = true) const;

    /**
     * Get all predecessor cells for a cell
     *
     * @param id ID of the cell
     */
    std::vector<std::string> predecessors(const std::string& id);

    /**
     * Get all successor cells for a cell
     *
     * @param id ID of the cell
     */
    std::vector<std::string> successors(const std::string& id);

    /**
     * Clear a cell
     *
     * After calling this method the cell will have no content and
     * no corresponding id (e.g. BD45) or name (e.g. total) in the spread.
     * To remove a name only from a cell `update()` it content
     *
     * @param id ID of the cell
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
     * A map of cell names to cell ids
     *
     * Used for normalising dependencies into ids only
     */
    std::map<std::string, std::string> names_;

    /**
     * Type for dependency graph
     *
     * Boost Graph Library does have a [`labelled_graph`](http://www.boost.org/doc/libs/1_58_0/boost/graph/labeled_graph.hpp)
     * which could potentially incorporate the functionality provided by the `cells_` map. But `labelled_graph` is not well documented
     * and seems to make working with BGL morre complicated. So for present, graph is used simply to provide a 
     * mapping of the dependency graph and not store cell data.
     *
     * Uses `boost::bidirectionalS` so that `boost::clear_in_edges` can be used in the `update()` method.
     * An altrnative is to use `boost::directedS` which takes less memory and to use some alternative approach 
     * in `update`. But this works.
     */
    typedef boost::adjacency_list<
        boost::listS, boost::vecS, boost::bidirectionalS,
        boost::property<boost::vertex_name_t, std::string>
    > Graph;

    typedef Graph::vertex_descriptor Vertex;

    /**
     * Variable (cell ids and aliases) dependency graph
     */
    Graph graph_;

    /**
     * A map of variable names to dependency graph vertex.
     *
     * Used to ease and quicken updates to the dependency tree
     */
    std::map<std::string, Graph::vertex_descriptor> vertices_;

    /**
     * Topological sort order of cells
     */
    std::vector<std::string> order_;

    /**
     * Has the dependency graph been initialised ?
     */
    bool prepared_ = false;

    /**
     * The current spread for this sheet
     */
    std::shared_ptr<Spread> spread_ = nullptr;

};

}  // namespace Stencila
