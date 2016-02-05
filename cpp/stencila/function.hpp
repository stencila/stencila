#pragma once

#include <array>
#include <map>
#include <string>
#include <vector>

#include <stencila/component.hpp>
#include <stencila/context.hpp>
#include <stencila/json.hpp>

namespace Stencila {

class Function : public Component {
 public:
    /**
     * @name Construction and destruction.
     * 
     * @{
     */

    Function(void);

    explicit Function(const std::string& from);

    ~Function(void);

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

    std::string name(void) const;
    Function& name(const std::string& name);

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
    Function& title(const std::string& title);

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


    struct Parameter {
        std::string name;
        std::string description;
    };
    std::vector<Parameter> parameters(void) const;

    void parameter(const Parameter& parameter);

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
    Function& initialise(const std::string& from);

    /**
     * Load this sheet from an input stream
     * 
     * @param  stream Input stream
     */
    Function& load(std::istream& stream, const std::string& format = "yaml");

    /**
     * Load this sheet from a string
     * 
     * @param  stream Input stream
     */
    Function& load(const std::string& string, const std::string& format = "yaml");

    /**
     * Dump this sheet to an output stream
     * 
     * @param  stream Output stream
     */
    const Function& dump(std::ostream& stream, const std::string& format = "yaml") const;

    /**
     * Dump this sheet to a string
     * 
     * @param  format Format for dump
     */
    std::string dump(const std::string& format = "yaml") const;

    /**
     * JSON getter and setter
     */
    Function& json(const std::string& content);
    std::string json(void) const;

    /**
     * Import this stencil content from a file
     * 
     * @param  path Filesystem path to file
     */
    Function& import(const std::string& path);

    /**
     * Export the stencil content to a file
     * 
     * @param  path Filesystem path to file
     */
    const Function& export_(const std::string& path) const;

    /**
     * Read this sheet from a directory
     * 
     * @param  path Filesystem path to a directory. 
     *              If an empty string then the sheet's current path is used.
     */
    Function& read(const std::string& path = "");

    /**
     * Write this sheet to a directory
     * 
     * @param  path Filesystem path to a directory
     *              If an empty string then the sheet's current path is used.
     */
    Function& write(const std::string& path = "");

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
    Function& compile(void);

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
    Function& view(void);

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

    // TODO move these into a common base class for
    // all "executable" components - Stencils, Sheets, Functions

    /**
     * Attach a context to this function
     *
     * @param context Context for execution
     */
    Function& attach(std::shared_ptr<Context> context);

    /**
     * Detach this functions's current context
     */
    Function& detach(void);


 private:

    std::string name_;

    std::string title_;

    std::vector<Parameter> parameters_;

    /**
     * The current context for this function
     */
    std::shared_ptr<Context> context_ = nullptr;
};

}  // namespace Stencila
