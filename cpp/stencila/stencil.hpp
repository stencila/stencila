#pragma once

#include <stencila/component.hpp>
#include <stencila/xml.hpp>
#include <stencila/context.hpp>

namespace Stencila {

class Stencil : public Component, public Xml::Document {
public:

    typedef Xml::Attribute Attribute;
    typedef Xml::Attributes Attributes;
    typedef Xml::Node Node;
    typedef Xml::Nodes Nodes;

    // Avoid ambiguities by defining which inherited method to use
    // when the base classes (Component & Xml::Document) use the same name
    using Component::path;
    using Component::destroy;

    Stencil(void){
    }

    Stencil(const std::string& from){
        initialise(from);
    }

    ~Stencil(void){
        if(context_) delete context_;
    }

    /**
     * @name Input and output
     *
     * Methods implemented in `stencil-io.cpp`
     * 
     * @{
     */

    /**
     * Initialise a stencil
     * 
     * @param  from A string indicating how the stencil is initialised
     */
    Stencil& initialise(const std::string& from);

    /**
     * Import the stencil content from a file
     * 
     * @param  path Filesystem path to file
     */
    Stencil& import(const std::string& path="");

    /**
     * Export the stencil content to a file
     * 
     * @param  path Filesystem path to file
     */
    Stencil& export_(const std::string& path="");

    /**
     * Read the stencil from a directory
     * 
     * @param  directory Filesystem path to a directory. 
     *                   If an empty string (`""`) then the stencil's current path is used.
     */
    Stencil& read(const std::string& directory="");
    
    /**
     * Write the stencil to a directory
     * 
     * @param  directory Filesystem path to a directory
     */
    Stencil& write(const std::string& directory="");

    /**
     * @}
     */

    /**
     * @name HTML parsing and generation
     *
     * Methods implemented in `stencil-html.cpp`
     * 
     * @{
     */
    
    /**
     * Get stencil content as HTML
     */
    std::string html(bool document = false, bool indent = true) const;

    /**
     * Set stencil content as HTML
     *
     * This method parses the supplied HTML, tidying it up in the process, 
     * and appends the resulting node tree to the stencil's XML tree
     * 
     * @param html A HTML string
     */
    Stencil& html(const std::string& html);

    /**
     * @}
     */
    

    /**
     * @name Cila parsing and generation
     *
     * Methods implemented in `stencil-cila.cpp`
     * 
     * @{
     */

    /**
     * Get stencil content as Cila
     */
    std::string cila(void) const;

    /**
     * Get stencil content as Cila written to an output stream
     *
     * @param stream Output stream to write to
     */
    std::ostream& cila(std::ostream& stream) const;

    /**
     * Set stencil content using Cila
     * 
     * @param cila A string of Cila code
     */
    Stencil& cila(const std::string& cila);

    /**
     * Set stencil content using Cila read from an input stream
     *
     * @param stream Input stream to read from
     */
    Stencil& cila(std::istream& stream);

    /**
     * @}
     */
    
    
    /**
     * @name Metadata
     *
     * Methods for obtaining metadata on a stencil.
     * At present only metadata getters are implemeted.
     * To set metadata add corresponding elements to the stencil's
     * HTML or Cila. 
     *
     * Methods implemented in `stencil-meta.cpp`
     * 
     * @{
     */

    /**
     * Get this stencil's title
     */
    std::string title(void) const;

    /**
     * Get this stencil's description
     */
    std::string description(void) const;

    /**
     * Get this stencil's keywords
     */
    std::vector<std::string> keywords(void) const;

    /**
     * Get this stencil's authors
     */
    std::vector<std::string> authors(void) const;
    
    /**
     * Get the list of rendering contexts that are compatible with this stencil.
     *
     * Context compatability will be determined by the expressions used in 
     * stencil directives like `data-with`,`data-text` etc. Some expressions
     * will be able to be used in multiple contexts.
     */
    std::vector<std::string> contexts(void) const;


    /**
     * Get this stencil's theme
     */
    std::string theme(void) const;


    /**
     * @name Rendering
     *
     * Methods implemented in `stencil-render.cpp`
     * 
     * @{
     */
    
    /**
     * Attach a context to this stencil
     *
     * @param context Context for rendering
     */
    Stencil& attach(Context* context);

    /**
     * Detach the stencil's current context
     *
     * The method `delete`s the context.
     */
    Stencil& detach(void);

    /**
     * Get details on this stencil's current context
     *
     * The method `delete`s the context.
     */
    std::string context(void) const;

    /**
     * Render an error onto a node.
     * 
     * A function for providing consistent error reporting from
     * within rendering functions.
     * 
     * @param node    Node where error occurred
     * @param type    Type of error, usually prefixed with directive type e.g. `for-syntax`
     * @param data    Data associated with the error which may be useful for resolving it
     * @param message A human readable description of the error
     */
    void render_error(Node node, const std::string& type, const std::string& data, const std::string& message);

    /**
     * Render a `code` element (e.g. `<code data-code="r,py">`)
     *
     * The text of the element is executed in the context if the context's type
     * is listed in the `data-code` attribute. If the context's type is not listed
     * then the element will not be rendered (i.e. will not be executed). 
     * 
     * This behaviour allows for polyglot stencils which have both `code` elements that
     * are either polyglot (valid in more than one languages) or monoglot (valid in only one language)
     * as required by similarities/differences in the language syntax e.g.
     *
     *    <code data-code="r,py">
     *        m = 1
     *        c = 299792458
     *    </code>
     * 
     *    <code data-code="r"> e = m * c^2 </code>
     *    <code data-code="py"> e = m * pow(c,2) </code>
     *    
     * 
     * `code` elements must have both the `code` tag and the `data-code` attribute.
     * Elements having just one of these will not be rendered.
     *
     * 
     *
     * This method is currently incomplete, it does not insert bitmap formats like PNG fully.
     * The best way to do that still needs to be worked out.
     */
    void render_code(Node node, Context* context);

    /**
     * Render a `set` element (e.g. `<span data-set="answer=42"></span>`)
     *
     * The expression in the `data-set` attribute is parsed and
     * assigned to a variable in the context.
     */
    std::string render_set(Node node, Context* context);

    /**
     * Render a `par` directive (e.g. `<span data-par="answer:number=42"></span>`)
     *
     * Returns the name,type and default value for the directive
     */
    std::array<std::string,3> render_par(Node node, Context* context,bool primary=true);

    /**
     * Render an `<input>` element (e.g. `<input name="answer" type="number" value="42"></input>`)
     */
    void render_input(Node node, Context* context);

    /**
     * Render a `text` element (e.g. `<span data-text="result"></span>`)
     *
     * The expression in the `data-text` attribute is converted to a 
     * character string by the context and used as the element's text.
     * If the element has a `data-off="true"` attribute then the element will not
     * be rendered and its text will remain unchanged.
     */
    void render_text(Node node, Context* context);

    /**
     * Render a `with` element (e.g. `<div data-with="sales"><span data-text="sum(quantity*price)" /></div>` )
     *
     * The expression in the `data-with` attribute is evaluated and made the subject of a new context frame.
     * All child nodes are rendered within the new frame. The frame is then exited.
     */
    void render_with(Node node, Context* context);

    /**
     * Render a `if` element (e.g. `<div data-if="answer==42">...</div>` )
     *
     * The expression in the `data-if` attribute is evaluated in the context.
     */
    void render_if(Node node, Context* context);

    /**
     * Render a `switch` element
     *
     * The first `case` element (i.e. having a `data-case` attribute) that matches
     * the `switch` expression is activated. All other `case` and `default` elements
     * are deactivated. If none of the `case` elements matches then any `default` elements are activated.
     */
    void render_switch(Node node, Context* context);

    /**
     * Render a `for` element `<ul data-for="planet:planets"><li data-text="planet" /></ul>`
     *
     * A `for` element has a `data-for` attribute which specifies the variable name given to each item and 
     * an expression providing the items to iterate over e.g. `planet:planets`. The variable name is optional
     * and defaults to "item".
     *
     * The first child element is rendered for each item and given a `data-index="<index>"`
     * attribute where `<index>` is the 0-based index for the item. If the `for` element has already been rendered and
     * already has a child with a corresponding `data-index` attribute then that is used, otherwise a new child is appended.
     * This behaviour allows for a user to `data-lock` an child in a `for` element and not have it lost. 
     * Any child elements with a `data-index` greater than the number of items is removed unless it has a 
     * descendent with a `data-lock` attribute in which case it is retained but marked with a `data-extra` attribute.
     */
    void render_for(Node node, Context* context);

    /**
     * Render an `include` element (e.g. `<div data-include="stats/t-test" data-select="macros text simple-paragraph" />` )
     */
    void render_include(Node node, Context* context);

    /**
     * Render the children of an HTML element
     * 
     * @param node    Node to render
     * @param context Context to render in
     */
    void render_children(Node node, Context* context);

    /**
     * Render a HTML element
     * 
     * @param node    Node to render
     * @param context Context to render in
     */
    void render(Node node, Context* context);

    /**
     * Render this stencil within a context
     * and attach the context.
     *
     * @param context Context for rendering
     */
    Stencil& render(Context* context);

    /**
     * Render this stencil in a new context
     * 
     * @param  type Type of context (e.g. "r", "py")
     */
    Stencil& render(const std::string& type);

    /**
     * Render this stencil, using the currenly attached context, or 
     * creating a new context if necessary
     */
    Stencil& render(void);
    
    /**
     * @}
     */
    
    /**
     * @name Serving
     *
     * Methods for serving a stencil over a nework.
     * Overrides of `Component` methods as required.
     *
     * Methods implemented in `stencil-serve.cpp`
     * 
     * @{
     */

    /**
     * Serve this stencil
     */
    std::string serve(void);

    /**
     * View this stencil
     */
    void view(void);

    /**
     * Execute a call on this stencil
     * 
     * @param  call A `Call` object
     */
    std::string call(const Call& call);

    /**
     * Generate a web page for a stencil
     */
    static std::string page(const Component* component);

    /**
     * Execute a call on a stencil
     */
    static std::string call(Component* component, const Call& call);

    /**
     * @}
     */

    /**
     * @name Sanitize
     * @{
     */
    
    static Xml::Whitelist whitelist;

    Stencil& sanitize(void) {
        //Xml::Document::sanitize(whitelist);
        return *this;
    };

    /**
     * @}
     */

    /**
     * Commit changes to this stencil
     * 
     * @param  message A commit message
     */
    Stencil& commit(const std::string& message=""){
        // Save the stencil..
        write();
        ///...then commit it
        Component::commit(message);
        return *this;
    }

private:

    /**
     * The current rendering context for this stencil
     */
    Context* context_ = nullptr;

    /**
     * A record of the number of elements of particular types within
     * this stencil
     */
    std::map<std::string,unsigned int> counts_;
};

/**
 * A list of tags allowed in a stencil
 */
#define STENCILA_STENCIL_TAGS (section)(nav)(article)(aside)(address)(h1)(h2)(h3)(h4)(h5)(h6)(p)(hr)(pre)(blockquote)(ol)(ul)(li)(dl)(dt)(dd)\
    (figure)(figcaption)(div)(a)(em)(strong)(small)(s)(cite)(q)(dfn)(abbr)(data)(time)(code)(var)(samp)(kbd)(sub)(sup)(i)(b)(u)(mark)(ruby)\
    (rt)(rp)(bdi)(bdo)(span)(br)(wbr)(ins)(del)(table)(caption)(colgroup)(col)(tbody)(thead)(tfoot)(tr)(td)(th)

/**
 * A list of [global attributes](http://www.w3.org/TR/html5/dom.html#global-attributes)(those that 
 * are "common to and may be specified on all HTML elements") and which are allowed in stencils.
 * Currenly this is a fairly restricted set. See above link for more that could be allowed.
 */
#define STENCILA_GLOBAL_ATTRS "class","id","lang","title","translate"

/**
 * A list of attributes that have semantic meaning in stencils
 */
#define STENCILA_DIRECTIVE_ATTRS "data-code","data-text","data-switch","data-case"

/**
 * Combination of the above two attribute lists
 */
#define STENCILA_STENCIL_ATTRS STENCILA_GLOBAL_ATTRS,STENCILA_DIRECTIVE_ATTRS

}
