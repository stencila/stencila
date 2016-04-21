#pragma once

#include <array>
#include <map>
#include <string>
#include <vector>

#include <boost/regex.hpp>
#include <boost/graph/adjacency_list.hpp>

#include <stencila/component.hpp>
#include <stencila/function.hpp>
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
     * Get the execution environment for this component
     *
     * Use "environ" rather than "language" because language
     * could later be used to describe the natural language e.g. "en"
     * and for any one programming language (e.g. Python) 
     * there may be more than one environment (e.g. py-2.7, py-3.4)
     */
    std::string environ(void) const;

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
     * Load this sheet from a string
     * 
     * @param  stream Input stream
     * @param  format Format of content (e.g. csv, xlsx)
     * @param  at     Cell to use as top-left
     */
    Sheet& load(const std::string& string, const std::string& format = "tsv", const std::string& at = "A1");

    /**
     * Load this sheet from an input stream
     * 
     * @param  stream Input stream
     * @param  format Format of content (e.g. tsv, csv)
     * @param  at     Cell to use as top-left
     */
    Sheet& load(std::istream& stream, const std::string& format = "tsv", const std::string& at = "A1");


    /**
     * Import this stencil content from a file
     * 
     * @param  path Filesystem path to file
     * @param  format Format of content (e.g. tsv, csv)
     * @param  at     Cell to use as top-left
     * @param  execute Should the imported cells be executed?
     */
    Sheet& import(const std::string& path, const std::string& at = "A1", bool execute = true);

    /**
     * Dump this sheet as script in host language
     * @param  stream Output stream
     */
    Sheet& dump_script(std::ostream& stream, const std::vector<std::string>& symbols);

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
     * Load separated values content into this sheet
     * 
     * @param  stream Input stream
     * @param  format Format of content (e.g. tsv, csv)
     * @param  at     Cell to use as top-left
     */
    Sheet& load_separated(std::istream& stream, const std::string& format, const std::string& at = "A1");

    /**
     * Dump separated values from this sheet
     * 
     * @param  stream Output stream
     * @param  format Format of content (e.g. tsv, csv)
     * @param  start  Cell to use as top-left
     * @param  end    Cell to use as bottom-right
     */
    Sheet& dump_separated(std::istream& stream, const std::string& format, const std::string& start = "A1", const std::string& end = "");

    /**
     * Load cells from an Office Open XML Spreadsheet (.xlsx) file into this sheet
     * 
     * @param  path Path to the .xlsx file
     * @param  sheet The sheet to load from
     * @param  at     Cell to use as top-left
     * @param  execute Should the loaded cells be executed?
     */
    Sheet& load_xlsx(const std::string& path, const std::string& sheet, const std::string& at, bool execute = true);

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
     * Read this sheet from a string
     */
    Sheet& read(const std::string& content, const std::string& format);

    /**
     * Write this sheet to a directory
     * 
     * @param  path Filesystem path to a directory
     *              If an empty string then the sheet's current path is used.
     */
    Sheet& write(const std::string& path = "");

    /**
     * Take a snapshot of this sheet
     */
    Sheet& store(void);

    /** 
     * Restore this sheet from the last available snapshot
     */
    Sheet& restore(void);

    /**
     * Generate a web page for a sheet
     *
     * @param  instance  A sheet instance
     */
    static std::string page(const Instance& instance);

    /**
     * Generate a web page for this sheet
     */
    std::string page(void) const;

    /**
     * Generate a web page for this sheet and write it to a file
     * (usually index.html) in it's working directory
     */
    Sheet& page(const std::string& filename);

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

    Wamp::Message message(const Wamp::Message& message);

    Json::Document call(const std::string& name, const Json::Document& args);

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
         * Identifier for this cell (e.g. D5)
         *
         * Cuttently, only used in `update()` method for returning a `vector`
         * of updates cells instead of a `map`
         */
        std::string id;

        /**
         * What kind of cell is this?
         *     
         * Alternative types of literal expression will be added in the future. Those may deal with literals that are
         * not necessarily be valid in the host language e.g. $1,000,000. The host context can be asked
         */
        enum Kind {
            // empty or blank (only whitespace) source
            blank_ = 0,

            // expression       eg. `= 22/7`
            expression_ = 1,
            // mapping          eg. `: my_matrix`
            mapping_ = 2,
            // requirement      eg. `^ library(ggplot2)`
            requirement_ = 3,
            // manual           eg. `| optim(...)`
            manual_ = 4,
            // test             eg. `? sum(A1:A10)==100`
            test_ = 5,
            // visualization    eg. `~ A1:B10 as points`
            visualization_ = 6,
            // Cila             eg. `_ The *slope*, |b|, equals |y_2-y_1//x_2-x_1|`
            cila_ = 7,

            // number literal eg. `42`
            number_ = 10,
            // string literal (single or double quoted) eg. `"foo"`
            string_ = 11,

            // text (default kind if nothing else matches)
            text_ = 255

        } kind = blank_;

        /**
         * Get a string code for the kind of the cell
         *
         * Convention is to use the first three letters of the kind.
         */
        std::string kind_string(void) const;

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
         *     A3: = A1*A2^2
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

        /**
         * Get the source for this cell
         */
        std::string source(void) const;

        /**
         * Set the source for this cell
         * 
         * @param  source New source
         */
        Cell& source(const std::string& source);

        /**
         * Get the display mode for this cell
         *
         * Either the display mode specified for this cell, or the
         * default display mode for the cell value type.
         */
        std::string display(void) const;

        /**
         * Get the display mode specified, if any, for this cell.
         */
        std::string display_specified(void) const;

        /**
         * Set the display mode for this cell
         *
         * @param  display New display
         */
        Cell& display(const std::string& display);

        /**
         * Specific display mode for this cell
         */
        std::string display_;
    };

    struct CellEmptyError : public Exception {
        using Exception::Exception;
    };


    /**
     * Get a cell from this sheet
     */
    Cell& cell(const std::string& id) throw(CellEmptyError);

    /**
     * Get a cell from this sheet
     */
    Cell& cell(unsigned int row, unsigned int col);

    /**
     * Get a cell pointer from this sheet
     *
     * Returns a `nullptr` if the cell does not exist
     * instead of raising an error like `cell` does
     */
    Cell* cell_pointer(const std::string& id);

    /**
     * Get a cell pointer from this sheet
     *
     * Returns a `nullptr` if the cell does not exist
     * instead of raising an error like `cell` does
     */
    Cell* cell_pointer(unsigned int row, unsigned int col);

    /**
     * Get cells from this sheets using an id (e.g. A1)
     * or range (e.g. A1:A10)
     */
    std::vector<Cell> cells(const std::string& range);

    /**
     * Set cells
     */
    Sheet& cells(const std::vector<std::array<std::string, 2>>& sources);

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
     * Regular expression used for cell identifiers (e.g. A1)
     */
    static boost::regex id_regex;

    /**
     * Regular expression used for cell ranges (e.g. A1:A10)
     */
    static boost::regex range_regex;

    /**
     * Is a string a valid cell identifier?
     */
    static bool is_id(const std::string& id);

    /**
     * Is a string a valid cell range?
     */
    static bool is_range(const std::string& range);

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
     * Generate the range row and column indices from a cell range
     */
    static std::array<unsigned int, 4> range(const std::string& range);

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
    static Cell parse(const std::string& content);

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
     * Update the sheet given changes to cells
     *
     * This method parses the new source and will then update the cells' corresponding
     * variable/s (both id and optional name) within the spread environment. Because of
     * interdependencies between cells this method is designed to take batches of cell updates,
     * analyse the dependency graph and then execute each cell expression. It returns a map of cells
     * whose `type` or `value` has changed (some cells may have changed in the spread but if their string
     * value has not changed then they will not be returned in the map)
     *
     * @param changes Map of cell IDs and their sources
     * @param execute Should cell expressions be executed inthe context?
     * @return List of IDs of the cells that have changed (including updated cells and their successors)
     */
    std::vector<Cell> update(const std::vector<Cell>& changes, bool execute = true);

    /**
     * Update a single cell with new source
     * 
     * @param  id      ID of the cell
     * @param  source  Cell source
     * @return         New value of the cell
     */
    std::vector<Cell> update(const std::string& id, const std::string& source);

    /**
     * Update a cell, or range of cells with existing source i.e. recaluclate
     * 
     * @param  range  Cell range or identifier
     */
    Sheet& update(const std::string& range);

    /**
     * Update all cells in this sheet
     *
     * This method might need to be called if for example a global variable
     * outside of the spread is altered
     */
    Sheet& update(bool execute = true);

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
    std::vector<std::string> order(void) const;

    /**
     * Generate a Graphviz `dot` file of the dependency graph for this sheet
     *
     * @param path  File system path for the `.dot` file. Defaults to `out/graph.dot`
     * @param image Should the `dot` program be called to produce a PNG?
     */
    void graphviz(const std::string& path = "", bool image = true) const;

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
     * Test this sheet
     *
     * Provide summary of tests and their coverage. Note that 
     * this does not actually run the test cells (that is done on
     * an as-needed basis using update), it just reports on them.
     *
     * @return A JSOn document containing test results
     */
    Json::Document test(void) const;

    /**
     * Clear all cells
     */
    Sheet& clear(void);

    /**
     * Get a list of functions that are available in this sheet's context
     */
    std::vector<std::string> functions(void) const;

    /**
     * Get a function definition from this sheet's context
     */
    Function function(const std::string& name) const;

 private:
    /**
     * Meta information on the sheet
     *
     * e.g. language, title, keywords
     */
    std::map<std::string, std::string> meta_;

    /**
     * Custom comparison to have column first ordering and row ordering
     * that is numeric, not string based (i.e "3" < "10")
     */
    struct IdComparison {
        bool operator()(const std::string& id1, const std::string& id2) const {
            auto in1 = Sheet::index(id1);
            auto in2 = Sheet::index(id2);
            // If same column, ...
            if (in1[1] == in2[1]) {
                // then compare row...
                return in1[0] < in2[0];
            }
            // otherwise, compare column
            return in1[1] < in2[1];
        }
    };

    /**
     * A map of cells having content
     */
    std::map<std::string, Cell, IdComparison> cells_;

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
    typedef Graph::edge_descriptor Edge;

    /**
     * Variable (cell ids and aliases) dependency graph
     */
    Graph graph_;

    /**
     * A map of variable names to dependency graph vertex.
     *
     * Used to ease and quicken updates to the dependency tree
     */
    std::map<std::string, Vertex> vertices_;

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


    friend class SheetGraphvizPropertyWriter;
};

}  // namespace Stencila
