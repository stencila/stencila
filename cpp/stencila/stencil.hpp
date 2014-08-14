#pragma once

#include <boost/lexical_cast.hpp>
#include <boost/preprocessor/seq/for_each.hpp>

#include <stencila/component.hpp>
#include <stencila/json.hpp>
#include <stencila/html.hpp>
// Conditional includes of context types
#if STENCILA_PYTHON_CONTEXT
    #include <stencila/python-context.hpp>
#endif
#if STENCILA_R_CONTEXT
    #include <stencila/r-context.hpp>
#endif

class Context;

namespace Stencila {

class Stencil : public Component, public Xml::Document {
public:
    
    // Avoid ambiguities by defining which inherited method to use
    // when the base classes use the same name
    using Component::path;
    using Component::destroy;

	typedef Xml::Attribute Attribute;
    typedef Xml::AttributeList AttributeList;
    typedef Xml::Node Node;
    typedef Xml::Nodes Nodes;

public:

    /**
     * @name Web interface methods
     *
     * Overrides of `Component` methods as required
     * 
     * @{
     */
    
    /**
     * Serve this stencil
     */
    std::string serve(void){
        return Component::serve(StencilCode);
    }

    /**
     * View this stencil
     */
    void view(void){
        return Component::view(StencilCode);
    }

    /**
     * Generate a web page for a stencil
     */
    static std::string page(const Component* component){
        return static_cast<const Stencil&>(*component).dump();
    }

    /**
     * Process a message for this stencil
     */
    static std::string call(Component* component, const Call& call){
        return static_cast<Stencil&>(*component).call(call);
    }

    std::string call(const Call& call) {
        auto what = call.what();
        
        // Getting content
        if(what=="html():string"){
            return html();
        }
        else if(what=="cila():string"){
            return cila();
        }

        // Setting content
        else if(what=="html(string)"){
            std::string string = call.arg(0);
            html(string);
        }
        else if(what=="cila(string)"){
            std::string string = call.arg(0);
            cila(string);
        }

        // Conversion of content...
        // ... HTML to Cila
        else if(what=="html(string).cila():string"){
            std::string string = call.arg(0);
            return     html(string).cila();
        }
        // ... Cila to HTML
        else if(what=="cila(string).html():string"){
            std::string string = call.arg(0);
            return     cila(string).html();
        }

        // Rendering...
        //... HTML
        else if(what=="html(string).render().html():string"){
            std::string string = call.arg(0);
            return     html(string).render().html();
        }
        //...Cila
        else if(what=="cila(string).render().cila():string"){
            std::string string = call.arg(0);
            return     cila(string).render().cila();
        }

        else return Component::call(call);

        return "";
    }

    /**
     * @}
     */

public:

    Stencil(void){
    }

    Stencil(const std::string& from){
        initialise(from);
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
     * @param  directory Filesystem path to a directory
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
     * @name Context
     * @{
     */
    
private:

    /**
     * The current rendering context for this stencil
     */
    Context* context_ = nullptr;

    /**
     * A list of rendering contexts that are compatible with this stencil.
     *
     * Context compatability will be determined by the expressions used in 
     * stencil directives like `data-with`,`data-text` etc. Some expressions
     * will be able to be used in multiple contexts.
     */
    std::vector<std::string> contexts_;

public:

    /**
     * Get the contexts that are supported by the stencil
     */
    const std::vector<std::string> contexts(void) const {
        return contexts_;
    }

    /**
     * Set the contexts that are supported by the stencil
     */
    Stencil& contexts(const std::vector<std::string>& values) {
        // Whitelist check of context values
        for(auto value : values){
            bool ok = false;
            for(auto context : {"py","r"}){
                if(value==context) ok = true;
            }
            if(not ok) STENCILA_THROW(Exception,"Context string not recognised: "+value)
        }
        contexts_ = values;
        return *this;
    }

    /**
     * @}
     */    
    
    /**
     * @name Theme
     * @{
     */
private:

    std::string theme_ = "core/stencils/themes/default";

public:

    const std::string& theme(void) const {
        return theme_;
    }

    Stencil& theme(const std::string& theme) {
        theme_ = theme;
        return *this;
    }

    /**
     * @}
     */

    /**
     * @name Rendering
     * @{
     */

    /**
     * Render this stencil within a context
     *
     * @param context Context for rendering
     */
    template<typename Context>
    Stencil& render(Context& context);

    /**
     * Render this stencil in a new context
     * 
     * @param  type Type of context (e.g. "r", "py")
     */
    Stencil& render(const std::string& type);

    /**
     * Render this stencil, creating a new context if necessary
     */
    Stencil& render(void);
    
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

    /**
     * @name Embedding
     * @{
     */
    
private:

    static std::stack<Node> embed_parents_;

public:

    Stencil& embed(void) {
        unembed();
        embed_parents_.push(*this);
        return *this;
    }

    Stencil& unembed(void) {
        embed_parents_ = std::stack<Node>();
        return *this;
    }

    static Node element(const std::string& tag, const AttributeList& attributes, const std::string& text = ""){
        // Append to the current parent
        Node parent = embed_parents_.top();
        Node elem = parent.append(tag,attributes,text);
        return elem;
    }

    static Node element(const std::string& tag, const std::string& text){
        return element(tag,AttributeList(),text);
    }

    static Node element(const std::string& tag){
        return element(tag,AttributeList());
    }

    template<typename... Args>
    static Node element(const std::string& tag, const AttributeList& attributes, Args... args){
        Node started = start(tag,attributes);
        add(args...);
        //Node finished = finish(tag);
        return started;
    }

    template<typename... Args>
    static Node element(const std::string& tag, Args... args){
        return element(tag,AttributeList(),args...);
    }

    static Node start(const std::string& tag, const AttributeList& attributes){
        Node elem = element(tag,attributes);
        embed_parents_.push(elem);
        return elem;
    }

    template<typename... Args>
    static void add(const std::string& text,Args... args){
        // Append a text node
        embed_parents_.top().append_text(text);
        add(args...);
    }

    template<typename... Args>
    static void add(Node& node,Args... args){
        // Append a node. This node must be moved from it's existing parent
        Node parent = embed_parents_.top();
        node.move(parent);
        add(args...);
    }

    template<typename... Args>
    static void add(void(*inner)(void),Args... args){
        // Execute the 
        inner();
        add(args...);
    }

    static void add(void){
    }

    static Node finish(const std::string& tag=""){
        Node elem = embed_parents_.top();
        embed_parents_.pop();
        return elem;
    }

    /**
     * @}
     */

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

// Declaration of "embedding" functions (definitons in `stencil.cpp`)
#define STENCILA_LOCAL(repeater,separator,tag)\
    Html::Node tag(void);\
    Html::Node tag(const std::string& text);\
    Html::Node tag(const Stencil::AttributeList& attributes, const std::string& text="");\
    Html::Node tag(void(*inner)(void));\
    template<typename... Args> Html::Node tag(const Stencil::AttributeList& attributes,Args... args);\
    template<typename... Args> Html::Node tag(Args... args);\
BOOST_PP_SEQ_FOR_EACH(STENCILA_LOCAL, ,STENCILA_STENCIL_TAGS)
#undef STENCILA_LOCAL

}
