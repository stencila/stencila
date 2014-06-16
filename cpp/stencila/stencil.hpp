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
        if(what=="html"){
            html(call.arg(0));
        }
        else if(what=="commit"){
            commit(call.arg(0));
        }
        else if(what=="render"){
            if(call.args()>0) html(call.arg(0));
            render();
            return html();
        }
        else {
            STENCILA_THROW(Exception,"Method not registered for calling: "+what);
        }
        return "";
    }

    /**
     * @}
     */

public:

    Stencil(void){
    }

    Stencil(const std::string& content){
        std::size_t found = content.find("://");
        if(found==std::string::npos) STENCILA_THROW(Exception,"Content type (e.g. html://, file://) not specified in supplied string")
        std::string type = content.substr(0,found);
        std::string rest = content.substr(found+3);
        if(type=="html") html(rest);
        else if(type=="file") read(rest);
        else STENCILA_THROW(Exception,"Unrecognised type: " + type)
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
        clear();
        Html::Document doc(html);
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
     * @name Node parsing methods
     *
     * Some stencil nodes require parsing of attributes or text
     * content to determine their semantics. These methods
     * provide for that parsing and are mostly used by other methods
     * such as `render`.
     * 
     * @{
     */

private:

    std::tuple<std::string,std::string> parse_param_or_set_(const Node& node, const std::string& attr){
        std::string value = node.attr(attr);
        std::string name;
        std::string expression;
        size_t semicolon = value.find(":");
        if(semicolon!=value.npos){
            name = value.substr(0,semicolon);
            expression = value.substr(semicolon+1);
        } else {
            name = value;
            expression = node.text();
        }
        return std::tuple<std::string,std::string>(name,expression);
    }

    std::tuple<std::string,std::string> parse_param_(const Node& node){
        return parse_param_or_set_(node,"data-param");
    }

    std::tuple<std::string,std::string> parse_set_(const Node& node){
        return parse_param_or_set_(node,"data-set");
    }

    /**
     * @}
     */
    

    /**
     * @name Rendering and display methods
     * @{
     */

public:

    /**
     * Render this stencil
     * 
     * @param  type Type of stencil (e.g. "r", "py")
     */
    Stencil& render(const std::string& type=""){
        // Get the list of context that are compatible with this stencil
        auto types = contexts();
        // Use the first in the list if type has not been specified
        std::string use;
        if(type.length()==0){
            if(types.size()==0){
                STENCILA_THROW(Exception,"No default context type for this stencil; please specify one.");
            }
            else use = types[0];
        } else {
            use = type;
        }
        // Render the stencil in the correposnding context type
        if(use=="py"){
            #if STENCILA_PYTHON_CONTEXT
                PythonContext context;
                render(context);
            #else
                STENCILA_THROW(Exception,"Stencila has not been compiled with support for Python contexts");
            #endif
        }
        else if(use=="r"){
            #if STENCILA_R_CONTEXT
                RContext context;
                render(context);
            #else
                STENCILA_THROW(Exception,"Stencila has not been compiled with support for R contexts");
            #endif
        }
        else {
           STENCILA_THROW(Exception,"Unrecognised context type: "+type); 
        }
        // Return self for chaining
        return *this;
    }

    /**
     * Render this stencil within an existing context
     *
     * @parameters context Context for rendering
     */
    template<typename Context>
    Stencil& render(Context& context){
        render_element_(*this,context);
        return *this;
    }
    
private:

    template<typename Context>
    void render_element_(Node node, Context& context){
        try {
            // Remove any existing errors
            for(Node child : node.children()){
                if(child.attr("data-error").length()>0){
                    node.remove(child);
                }
            }
            
            // Check for handled elements
            // For each attribute in this node...
            //...use the name of the attribute to dispatch to another rendering method
            //   Note that return is used so that only the first Stencila "data-xxx" will be 
            //   considered and that directive will determine how/if children nodes are processed
            std::string tag = node.name();
            for(std::string attr : node.attrs()){
                // `macro` elements are not rendered
                if(attr=="data-macro") return ;
                else if(attr=="data-code") return render_code_(node,context);
                else if(attr=="data-text") return render_text_(node,context);
                else if(attr=="data-image") return render_image_(node,context);
                else if(attr=="data-with") return render_with_(node,context);
                else if(attr=="data-if") return render_if_(node,context);
                // Ignore `elif` and `else` elements as these are processed by `render_if_`
                else if(attr=="data-elif" or attr=="data-else") return;
                else if(attr=="data-switch") return render_switch_(node,context);
                else if(attr=="data-for") return render_for_(node,context);
                else if(attr=="data-include") return render_include_(node,context);
            }
            // If return not yet hit then process children of this element
            render_children_(node,context);
        }
        catch(std::exception& exc){
            node.append("span",{{"data-error","exception"}},std::string("Error:")+exc.what());
        }
        catch(...){
            node.append("span",{{"data-error","unknown"}},"Unknown error");
        }
    }

    template<typename Context>
    void render_children_(Node node, Context& context){
        for(Node child : node.children()) render_element_(child,context);
    }

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
     */
    template<typename Context>
    void render_code_(Node node, Context& context){
        // Get the list of contexts and ensure this context is in the list
        std::string contexts = node.attr("data-code");
        std::vector<std::string> items;
        boost::split(items,contexts,boost::is_any_of(","));
        bool ok = false;
        for(std::string& item : items){
            boost::trim(item);
            if(context.accept(item)){
                ok = true;
                break;
            }
        }
        // If ok, execute the code, otherwise just ignore
        if(ok){
            std::string code = node.text();
            context.execute(code);
        }
    }

    /**
     * Render a `text` element (e.g. `<span data-text="result"></span>`)
     *
     * The expression in the `data-text` attribute is converted to a 
     * character string by the context and used as the element's text.
     * If the element has a `data-off="true"` attribute then the element will not
     * be rendered and its text will remain unchanged.
     */
    template<typename Context>
    void render_text_(Node node, Context& context){
        if(node.attr("data-lock")!="true"){
            std::string expression = node.attr("data-text");
            std::string text = context.write(expression);
            node.text(text);
        }
    }

    /** 
     * Render a `image` element (e.g `<code data-image="svg">plot(x,y)</code>`)
     *
     * `image` elements capture any images produced by executing the enclosed code
     * in the context. `image` elements can be of alternative graphic formats e.g `svg`,`png`
     * When the code of a `image` element is sucessfully executed a child node is appended
     * which contains the resulting image and has the `data-data="true"` attribute.
     *
     * This method is currently incomplete, it does not insert bitmap formats like PNG fully.
     * The best way to do that still needs to be worked out.
     */
    template<typename Context>
    void render_image_(Node node, Context& context){
        std::string format = node.attr("data-image");
        std::string code = node.text();
        std::string image = context.paint(format,code);
        if(format=="svg"){
            Node svg = node.append_xml(image);
            svg.attr("data-data","true");
        } 
        else if(format=="png"){
            node.append("img",{
                {"src",""},
                {"data-data","true"}
            });
        }
        else {
            node.append(
                "div",
                {{"data-error","image-format"},{"data-format",format}},
                "Image format not recognised: "+format
            );
        }
    }

    /**
     * Render a `with` element (e.g. `<div data-with="sales"><span data-text="sum(quantity*price)" /></div>` )
     *
     * The expression in the `data-with` attribute is evaluated and made the subject of a new context frame.
     * All child nodes are rendered within the new frame. The frame is then exited.
     */
    template<typename Context>
    void render_with_(Node node, Context& context){
        std::string expression = node.attr("data-with");
        context.enter(expression);
        render_children_(node,context);
        context.exit();
    } 

    /**
     * Render a `if` element (e.g. `<div data-if="answer==42">...</div>` )
     *
     * The expression in the `data-if` attribute is evaluated in the context.
     */
    template<typename Context>
    void render_if_(Node node, Context& context){
        std::string expression = node.attr("data-if");
        bool hit = context.test(expression);
        if(hit){
            node.erase("data-off");
            render_children_(node,context);
        } else {
            node.attr("data-off","true");
        }
        // Iterate through sibling elements to turn them on or off
        // if they are elif or else elements; break otherwise.
        Node next = node.next_element();
        while(next){
            if(next.has("data-elif")){
                if(hit){
                    next.attr("data-off","true");
                } else {
                    std::string expression = next.attr("data-elif");
                    hit = context.test(expression);
                    if(hit){
                        next.erase("data-off");
                        render_children_(next,context);
                    } else {
                        next.attr("data-off","true");
                    }
                }
            }
            else if(next.has("data-else")){
                if(hit){
                    next.attr("data-off","true");
                } else {
                    next.erase("data-off");
                    render_children_(next,context);
                }
                break;
            }
            else break;
            next = next.next_element();
        }
    }

    /**
     * Render a `switch` element
     *
     * The first `case` element (i.e. having a `data-case` attribute) that matches
     * the `switch` expression is activated. All other `case` and `default` elements
     * are deactivated. If none of the `case` elements matches then any `default` elements are activated.
     */
    template<typename Context>
    void render_switch_(Node node, Context& context){
        std::string expression = node.attr("data-switch");
        context.mark(expression);

        bool matched = false;
        for(Node child : node.children()){
            if(child.has("data-case")){
                if(matched){
                    child.attr("data-off","true");
                } else {
                    std::string match = child.attr("data-case");
                    matched = context.match(match);
                    if(matched){
                        child.erase("data-off");
                        render_element_(child,context);
                    } else {
                        child.attr("data-off","true");
                    }
                }
            }
            else if(child.has("data-default")){
                if(matched){
                    child.attr("data-off","true");
                } else {
                    child.erase("data-off");
                    render_element_(child,context);
                }
            } else {
                render_element_(child,context);
            }
        }

        context.unmark();
    }

    /**
     * Render a `for` element `<ul data-for="planet:planets"><li data-each data-text="planet" /></ul>`
     *
     * A `for` element has a `data-for` attribute which specifies the variable name given to each item and 
     * an expression providing the items to iterate over e.g. `planet:planets`. The variable name is optional
     * and defaults to "item".
     *
     * The child element having a `data-each` attribute is rendered for each item and given a `data-index="<index>"`
     * attribute where `<index>` is the 0-based index for the item. If the `for` element has already been rendered and
     * already has a child with a corresponding `data-index` attribute then that is used, otherwise a new child is appended.
     * This behaviour allows for a user to `data-lock` an child in a `for` element and not have it lost. 
     * Any child elements with a `data-index` greater than the number of items is removed unless it has a 
     * descendent with a `data-lock` attribute in which case it is retained but marked with a `data-extra` attribute.
     */
    template<typename Context>
    void render_for_(Node node, Context& context){
        std::string parts = node.attr("data-for");
        // Get the name of item and items
        std::string item = "item";
        std::string items;
        std::vector<std::string> bits;
        boost::split(bits,parts,boost::is_any_of(":"));
        if(bits.size()==1){
            items = bits[0];
        } else if(bits.size()==2){
            item = bits[0];
            items = bits[1];
        } else {
            throw Exception("Error in parsing for item and items; more than one semicolon (:).");
        }

        // Initialise the loop
        bool more = context.begin(item,items);
        // Get the `data-each` node
        Node each = node.one("[data-each]");
        // If is for loop has been rendered before then `each` will have a `data-off`
        // attribute. So erase that attribute so that the repeated nodes don't get it
        if(each) each.erase("data-off");
        // Iterate
        int count = 0;
        while(each and more){
            // See if there is an existing child with a corresponding `data-index`
            std::string index = boost::lexical_cast<std::string>(count);
            Node item = node.one("[data-index=\""+index+"\"]");
            if(item){
                // If there is check to see if it is locked
                Node locked = item.one("[data-lock]");
                if(not locked){
                    // If it is then destory and replace it
                    item.destroy();
                    item = node.append(each);
                }
            } else {
                // If there is not, create one
                item = node.append(each);
            }
            // Erase and set index as required
            item.erase("data-each");
            item.attr("data-index",index);
            // Render the element
            render_element_(item,context);
            // Ask context to step to next item
            more = context.next();
            count++;
        }
        // Deactivate the each object
        if(each) each.attr("data-off","true");
        // Remove any children having a `data-index` attribute greater than the 
        // number of items, unless it has a `data-lock` decendent
        Nodes indexeds = node.all("[data-index]");
        for(Node indexed : indexeds){
            std::string index_string = indexed.attr("data-index");
            int index = boost::lexical_cast<int>(index_string);
            if(index>count-1){
                Node locked = indexed.one("[data-lock]");
                if(locked){
                    indexed.attr("data-extra","true");
                    // Move the end of the `for` element
                    indexed.move(node);
                }
                else indexed.destroy();
            }
        }
    }

    /**
     * Render an `include` element (e.g. `<div data-include="stats/t-test" data-select="macros text simple-paragraph" />` )
     */
    template<typename Context>
    void render_include_(Node node, Context& context){
        std::string include = node.attr("data-include");
        std::string version = node.attr("data-version");
        std::string select = node.attr("data-select");

        // If this node has been rendered before then there will be 
        // a `data-included` node that needs to be cleared first. If it
        // does not yet exist then append it.
        Node included = node.one("[data-included]");
        if(included){
            // If this node has been edited then it may have a data-lock
            // element. If it does then do NOT overwrite the exisiting contents
            // and simply return straight away.
            Node lock = included.one("[data-lock=\"true\"]");
            if(lock) {
                return;
            } else {
                included.clear();
            }
        }
        else included = node.append("div",{{"data-included","true"}});
        
        //Obtain the included stencil...
        Node stencil;
        //Check to see if this is a "self" include, otherwise obtain the stencil
        if(include==".") stencil = node.root();
        else stencil = get<Stencil>(include,version);
        // ...select from it
        if(select.length()>0){
            // ...append the selected nodes.
            for(Node node : stencil.all(select)){
                // Append the node first to get a copy of it which can be modified
                Node appended = included.append(node);
                // Remove `macro` declaration if any so that element gets rendered
                appended.erase("data-macro");
                // Remove "id=xxxx" attribute if any to prevent duplicate ids in a single document (http://www.w3.org/TR/html5/dom.html#the-id-attribute; although many browsers allow it)
                // This is particularly important when including a macro with an id. If the id is not removed, subsequent include elements which select for the same id to this one will end up
                // selecting all those instances where the macro was previously included.
                appended.erase("id");
            }
        } else {
            // ...append the entire stencil. No attempt is made to remove macros when included an entire stencil.
            included.append(stencil);
        }

        //Apply modifiers
        const int modifiers = 7;
        enum {
            delete_ = 0,
            replace = 1,
            change = 2,
            before = 3,
            after = 4,
            prepend = 5,
            append = 6
        };
        std::string attributes[modifiers] = {
            "data-delete",
            "data-replace",
            "data-change",
            "data-before",
            "data-after",
            "data-prepend",
            "data-append"
        };
        for(int type=0;type<modifiers;type++){
            std::string attribute = attributes[type];
            for(Node modifier : node.all("["+attribute+"]")){
                std::string selector = modifier.attr(attribute);
                for(Node target : included.all(selector)){
                    Node created;
                    switch(type){

                        case delete_:
                            target.destroy();
                        break;

                        case change:
                            target.clear();
                            target.append_children(modifier);
                        break;

                        case replace: 
                            created = target.before(modifier);
                            target.destroy();
                        break;
                        
                        case before:
                            created = target.before(modifier);
                        break;
                        
                        case after:
                            created = target.after(modifier);
                        break;
                        
                        case prepend:
                            created = target.prepend(modifier);
                        break;
                        
                        case append:
                            created = target.append(modifier);
                        break;
                    }
                    // Remove the modifier attribute from any newly created node
                    if(created) created.erase(attribute);
                }
            }
        }
        
        // Enter a new namespace.
        // Do this regardless of whether there are any 
        // `data-param` elements, to avoid the included elements polluting the
        // main context or overwriting variables inadvertantly
        context.enter();

        // Apply `data-set` elements
        // Apply all the `set`s specified in this include first. This
        // my include params not specified by the author of the included stencil.
        std::vector<std::string> assigned;
        for(Node set : node.all("[data-set]")){
            // Parse the parameter node
            std::tuple<std::string,std::string> parsed = parse_set_(set);
            std::string name = std::get<0>(parsed);
            std::string expression = std::get<1>(parsed);
            // Assign the parameter in the new frame
            context.assign(name,expression);
            // Add this to the list of parameters assigned
            assigned.push_back(name);
        }
        // Now apply the included element's parameters
        // Check for if they are required or for any default values
        for(Node param : included.all("[data-param]")){
            // Parse the parameter node
            std::tuple<std::string,std::string> parsed = parse_param_(param);
            std::string name = std::get<0>(parsed);
            std::string expression = std::get<1>(parsed);
            // Check to see if it has already be assigned
            if(std::count(assigned.begin(),assigned.end(),name)==0){
                if(expression.length()>0){
                    // Assign the parameter in the new frame
                    context.assign(name,expression);
                } else {
                    // Set an error
                    included.append(
                        "div",
                        {{"data-error","param-required"},{"data-param",name}},
                        "Parameter is required because it has no default: "+name
                    );
                }
            }
            // Remove the parameter, there is no need to have it in the included node
            param.destroy();
        }

        // Render the `data-included` element
        render_children_(included,context);
        
        // Exit the included node
        context.exit();
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
     * @name Sanitize
     * @{
     */
    
    static Xml::Whitelist whitelist;

    Stencil& sanitize(void) {
        Xml::Document::sanitize(whitelist);
    };

    /**
     * @}
     */


    /**
     * Read the stencil from a directory
     * 
     * @param  from Filesystem path to directory
     */
    Stencil& read(const std::string& from=""){
        // Call base read method to set `path`
        Component::read(from);
        
        Html::Document doc;
        doc.read(from);
        
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
                items.push_back(item.text());
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

        // Sanitize before proceeding
        sanitize();

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
         */
        body.append("script",{
            {"src","/core/themes/boot.js"}
        }," ");
        body.append("script",{
            {"src","/" + theme() + "/theme.js"}
        }," ");

        return doc.dump();
    }

    /**
     * Write the stencil to a directory
     * 
     * @param  to Filesystem path to directory
     */
    Stencil& write(std::ostream& stream){
        stream<<dump();
        return *this;
    }
    
    /**
     * Write the stencil to a directory
     * 
     * @param  to Filesystem path to directory
     */
    Stencil& write(const std::string& to=""){
        // Call base write method to set `path`
        Component::write(to);
        // Write to file
        std::ofstream file(to);
        write(file);
        file.close();

        return *this;
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
        Node finished = finish(tag);
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
