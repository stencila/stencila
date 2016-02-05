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
     * Methods for obtaining attributes of the function.
     * 
     * @{
     */

    /**
     * Get the component type for functions
     */
    static Component::Type type(void);

    std::string name(void) const;
    Function& name(const std::string& name);


    /**
     * Get this function's title
     */
    std::string title(void) const;
    Function& title(const std::string& title);


    /**
     * Get this function's summary
     */
    std::string summary(void) const;
    Function& summary(const std::string& summary);


    /**
     * Get this function's description.
     * 
     * A simple alias for `summary` provided to meet
     * the `Component` interface (e.g for use in page generation)
     */
    std::string description(void) const {
        return summary();
    }


    /**
     * Get this function's details
     */
    std::string details(void) const;
    Function& details(const std::string& details);


    /**
     * Get this function's keywords
     */
    std::vector<std::string> keywords(void) const;
    Function& keywords(const std::vector<std::string>& keywords);

    /**
     * Get this functions's authors
      */
    std::vector<std::string> authors(void) const;
    Function& authors(const std::vector<std::string>& authors);


    struct Parameter {
        std::string name;
        std::string description;
    };
    std::vector<Parameter> parameters(void) const;

    void parameter(const Parameter& parameter);

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
     * Initialise this function
     * 
     * @param  from A string indicating how the function is initialised
     */
    Function& initialise(const std::string& from);

    /**
     * Load this function from an input stream
     * 
     * @param  stream Input stream
     */
    Function& load(std::istream& stream, const std::string& format = "yaml");

    /**
     * Load this function from a string
     * 
     * @param  stream Input stream
     */
    Function& load(const std::string& string, const std::string& format = "yaml");

    /**
     * Dump this function to an output stream
     * 
     * @param  stream Output stream
     */
    const Function& dump(std::ostream& stream, const std::string& format = "yaml") const;

    /**
     * Dump this function to a string
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
     * Read this function from a directory
     * 
     * @param  path Filesystem path to a directory. 
     *              If an empty string then the function's current path is used.
     */
    Function& read(const std::string& path = "");

    /**
     * Write this function to a directory
     * 
     * @param  path Filesystem path to a directory
     *              If an empty string then the function's current path is used.
     */
    Function& write(const std::string& path = "");

    /**
     * Generate a web page for a function
     *
     * @param  component  A pointer to a function
     */
    static std::string page(const Component* component);

    /**
     * Generate a web page for this function
     */
    std::string page(void) const;

    /**
     * Compile this function
     *
     * Export this function as HTML to `index.html` in home directory
     */
    Function& compile(void);

    /**
     * @}
     */


    /**
     * @name Serving
     *
     * Methods for serving a function over a network.
     * Overides of `Component` methods as required.
     *
     * @{
     */

    /**
     * Serve this function
     */
    std::string serve(void);

    /**
     * View this function
     */
    Function& view(void);

    /**
     * Respond to a web request to a function
     *
     * @param  component  A pointer to a function
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
     * Respond to a web request to this function
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

    std::string summary_;

    std::vector<std::string> keywords_;

    std::string details_;

    std::vector<std::string> authors_;

    std::vector<Parameter> parameters_;

    /**
     * The current context for this function
     */
    std::shared_ptr<Context> context_ = nullptr;
};

}  // namespace Stencila
