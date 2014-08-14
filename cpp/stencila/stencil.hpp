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
     * Initialise a stencil
     * 
     * @param  from A string indicating how the stecnil is initialised
     */
    Stencil& initialise(const std::string& from){
        std::size_t found = from.find("://");
        if(found==std::string::npos){
            // Initialised from a path
            read(from);
        } else {
            // Initialised from some content
            std::string type = from.substr(0,found);
            std::string content = from.substr(found+3);
            if(type=="html") html(content);
            else if(type=="cila") cila(content);
            else if(type=="file") import(content);
            else STENCILA_THROW(Exception,"Unrecognised content type: " + type);
        }
    }

    /**
     * Get the content of the stencil
     * 
     * @param  format Format for content
     */
    std::string content(const std::string& format="html") const {
        if(format=="html") return html();
        else if(format=="cila") return cila();
        else STENCILA_THROW(Exception,"Format code not recognised: "+format);
    }

    /**
     * Set the content of the stencil
     *
     * @param  format Format for content
     */
    Stencil& content(const std::string& format, const std::string& content){
        if(format=="html") html(content);
        else if(format=="cila") cila(content);
        else STENCILA_THROW(Exception,"Format code not recognised: "+format);
        return *this;
    }

    /**
     * Get stencil content as HTML
     */
    std::string html(void) const {
        // Dump content as html.
        // Note that this explicitly avoids Stencil::dump
        // and uses indentation
        return Xml::Document::dump(true);
    }

    /**
     * Set stencil content as HTML
     *
     * This method parses the supplied HTML, tidying it up in the process, and appends the resulting node tree
     * to the stencil's XML tree
     * 
     * @param html A HTML string
     */
    Stencil& html(const std::string& html){
        Html::Document doc(html);

        // Being a valid HTML5 document, doc already has a <head> <title> and <body>
        // so these do not have to be checked for
        Node head = doc.find("head");
        Node body = doc.find("body");

        // Title
        title(head.find("title").text());

        // Keywords
        if(Node elem = head.find("meta","name","keywords")){
            std::string content = elem.attr("content");
            std::vector<std::string> items;
            boost::split(items,content,boost::is_any_of(","));
            for(auto& keyword : items) boost::trim(keyword);
            keywords(items);
        }

        // Description
        if(Node elem = head.find("meta","name","description")){
            std::string content = elem.attr("content");
            description(content);
        }       

        // Contexts
        if(Node elem = body.find("ul","id","contexts")){
            std::vector<std::string> items;
            for(auto& item : elem.all("li")){
                std::string context = item.text();
                boost::trim(context);
                if(context.length()) items.push_back(context);
            }
            contexts(items);  
        }

        // Authors
        if(Node elem = body.find("address","id","authors")){
            std::vector<std::string> items;
            for(auto& item : elem.all("a[rel=\"author\"]")){
                items.push_back(item.text());
            }
            authors(items);  
        }

        // Content
        // Clear content before appending new content from Html::Document
        clear();
        if(Node elem = body.find("main","id","content")){
            append_children(elem);
        }
        append_children(doc.find("body"));
        return *this;
    }

    /**
     * @name Cila parsing and generating
     *
     * Method implementation in `stencil-cila.cpp`
     * 
     * @{
     */

public:

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
     * @}
     */

    /**
     * @name Contexts
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
     * @name Rendering methods
     * @{
     */

public:

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
     * Import the stencil content from a file
     * 
     * @param  path Filesystem path to file
     */
    Stencil& import(const std::string& path=""){
        std::string ext = boost::filesystem::extension(path);
        if(ext==".html" or ext==".cila"){
            std::ifstream file(path);
            std::stringstream stream;
            stream<<file.rdbuf();
            std::string content = stream.str();
            if(ext==".html") html(content); 
            else if(ext==".cila") cila(content);
        }
        else STENCILA_THROW(Exception,str(boost::format("File extension <%s> not valid for a Stencil")%ext));
        return *this;
    }

    /**
     * Export the stencil content to a file
     * 
     * @param  path Filesystem path to file
     */
    Stencil& export_(const std::string& path=""){
        std::string ext = boost::filesystem::extension(path);
        if(ext==".html" or ext==".cila"){
            std::ofstream file(path);
            if(ext==".html") file<<html(); 
            else if(ext==".cila") file<<cila();
        }
        else STENCILA_THROW(Exception,str(boost::format("File extension <%s> not valid for a Stencil")%ext));
        return *this;
    }

    /**
     * Read the stencil from a directory
     * 
     * @param  directory Filesystem directory to directory
     */
    Stencil& read(const std::string& directory=""){
        if(directory.length()){
            // Check that directory exits and is a directory
            if(not boost::filesystem::exists(directory)){
                STENCILA_THROW(Exception,str(boost::format("Path <%s> does not exist")%directory));
            }
            if(not boost::filesystem::is_directory(directory)){
                STENCILA_THROW(Exception,str(boost::format("Path <%s> is not a directory")%directory));
            }
            // Set the stencil's path
            path(directory);
        }
        // Currently, set the stencil's content from main.cila
        boost::filesystem::path cila = boost::filesystem::path(directory) / "main.cila";
        if(not boost::filesystem::exists(cila)){
            STENCILA_THROW(Exception,str(boost::format("Directory <%s> does contain a 'main.cila' file")%directory));
        }
        import(cila.string());
        return *this;
    }
    
    /**
     * Write the stencil to a directory
     * 
     * @param  path Filesystem path
     */
    Stencil& write(const std::string& directory=""){
        // Set `path` if provided
        if(directory.length()) Component::path(directory);
        // Write necessary files
        // @fixme This should use `export_()`
        Component::write("main.html",dump());
        return *this;
    }

    /**
     * Dump the stencil to a HTML document string
     */
    std::string dump(void) const {
        Html::Document doc;

        // Being a valid HTML5 document, doc already has a <head> <title> and <body>
        Node head = doc.find("head");
        Node body = doc.find("body");

        // Title to <title>
        // Although we are creating an XHTML5 document, if the title tag is empty (i.e <title />)
        // this can cause browser parsing errors. So always ensure that there is some title content.
        auto t = title();
        if(t.length()==0) t = "Untitled";
        head.find("title").text(t);

        // Keywords to <meta>
        auto k = keywords();
        if(k.size()>0){
            head.append("meta",{
                {"name","keywords"},
                {"content",boost::algorithm::join(k,", ")}
            });
        }

        // Description to <meta>
        auto d = description();
        if(d.length()>0){
            head.append("meta",{
                {"name","description"},
                {"content",d}
            });
        }

        // The following tags are added with a space as content so that they do not
        // get rendered as empty tags (e.g. <script... />). Whilst empty tags should be OK with XHTML
        // they can cause problems with some browsers.

        /**
         * <link rel="stylesheet" ...
         *
         * Links to CSS stylesheets are [placed in the head](http://developer.yahoo.com/performance/rules.html#css_top) 
         * Use a site-root relative path (by adding the leading forward slash) so that CSS can be served from 
         * localhost.
         */
        std::string css = "/" + theme() + "/theme.css";
        head.append("link",{
            {"rel","stylesheet"},
            {"type","text/css"},
            {"href",css}
        }," ");

        /**
         * #contexts
         *
         * Added as a <ul> in body
         */
        auto c = contexts();
        if(c.size()>0){
            Node contexts_elem = body.append("ul",{
                {"id","contexts"}
            }," ");
            for(auto context : c){
                contexts_elem.append("li",{
                    {"class",context}
                },context);
            }
        }

        /**
         * #authors
         *
         * Use both <address> and <a rel="author" ...> as suggested at http://stackoverflow.com/a/7295013 .
         * The placement of <address> as a child of <body> should mean that this authors list applies to the whole document.
         * See:
            http://html5doctor.com/the-address-element/
            http://www.w3.org/TR/html5/sections.html#the-address-element
            http://stackoverflow.com/questions/7290504/which-html5-tag-should-i-use-to-mark-up-an-authors-name
         */
        auto a = authors();
        if(a.size()>0){
            auto authors_elem = body.append("address",{
                {"id","authors"}
            }," ");
            for(auto author : authors()){
                authors_elem.append("a",{
                    {"rel","author"},
                    {"href","#"}
                },author);
            }
        }

        /**
         * #content
         *
         * Content is placed in a <main> rather than just using the <body> so that extra HTML elements can be added by the 
         * theme without affecting the stencil's content.
         */
        auto content = body.append("main",{
            {"id","content"}
        }," ");
        content.append(*this);

        /**
         * <script>
         *
         * Script elements are [placed at bottom of page](http://developer.yahoo.com/performance/rules.html#js_bottom)
         * Files are with a fallback to hub.
         */
        body.append("script",{{"src","/core/themes/boot.js"}}," ");
        body.append("script","if(!window.Stencila){window.StencilaHost='http://hub.stenci.la';document.write(unescape('%3Cscript src=\"http://hub.stenci.la/core/themes/boot.js\"%3E%3C/script%3E'))}");
        body.append("script","window.Stencila.Booter.theme('" + theme() + "');");

        return doc.dump();
    }

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

#include <stencila/stencil-render.hpp>
