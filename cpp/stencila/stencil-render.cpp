#include <boost/regex.hpp>

#include <stencila/stencil.hpp>

// Conditional includes of context types
#if STENCILA_PYTHON_CONTEXT
    #include <stencila/python-context.hpp>
#endif
#if STENCILA_R_CONTEXT
    #include <stencila/r-context.hpp>
#endif

namespace Stencila {

// Anonymous namespace to contain rendering helper functions
namespace {

typedef Xml::Attribute Attribute;
typedef Xml::AttributeList AttributeList;
typedef Xml::Node Node;
typedef Xml::Nodes Nodes;

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
void error_(Node node, const std::string& type, const std::string& data, const std::string& message){
    node.append(
        // A <div> with...
        "div",
        // attributes...
        {
            // to identify the type of error,
            {"data-error",type},
            // and data for helping resolve the error...
            {"data-"+type,data}
        },
        // and a message as text content
        message
    );
}

// Forward declaration of the element rendering function
void element_(Node node, Context* context);

/**
 * Render all the children of a node
 */
void children_(Node node, Context* context){
    for(Node child : node.children()) element_(child,context);
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
 *
 * 
 *
 * This method is currently incomplete, it does not insert bitmap formats like PNG fully.
 * The best way to do that still needs to be worked out.
 */
void code_(Node node, Context* context){
    // Get the list of contexts and ensure this context is in the list
    std::string contexts = node.attr("data-code");
    std::vector<std::string> items;
    boost::split(items,contexts,boost::is_any_of(","));
    bool ok = false;
    for(std::string& item : items){
        boost::trim(item);
        if(context->accept(item)){
            ok = true;
            break;
        }
    }
    // If ok, execute the code, otherwise just ignore
    if(ok){
        // Get code and format etc
        std::string code = node.text();
        if(code.length()>0){
            std::string format = node.attr("data-format");
            std::string size = node.attr("data-size");
            std::string width,height,units;
            if(size.length()){
                boost::regex regex("([0-9]*\\.?[0-9]+)x([0-9]*\\.?[0-9]+)(cm|in|px)?");
                boost::smatch matches;
                if(boost::regex_match(size, matches, regex)){
                    width = matches[1];
                    height = matches[2];
                    if(matches.size()>2) units = matches[3];
                }
            }
            // Execute
            std::string output = context->execute(code,format,width,height,units);
            // Remove any existing output
            Node next = node.next_element();
            if(next and next.attr("data-output")=="true") next.destroy();
            // Append new output
            if(format.length()){
                Xml::Document doc;
                Node output_node;
                if(format=="out"){
                    output_node = doc.append("samp",output);
                }
                else if(format=="png" or format=="svg"){
                    output_node = doc.append("img",{
                        {"src",output}
                    });
                }
                else {
                    output_node = doc.append(
                        "div",
                        {{"data-error","output-format"},{"data-format",format}},
                        "Output format not recognised: "+format
                    );
                }
                // Flag output node 
                output_node.attr("data-output","true");
                // Create a copy immeadiately after code directive
                node.after(output_node);
            }
        }
    }
}

/**
 * Render a `set` element (e.g. `<span data-set="answer=42"></span>`)
 *
 * The expression in the `data-set` attribute is parsed and
 * assigned to a variable in the context.
 */
std::string set_(Node node, Context* context){
    std::string attribute = node.attr("data-set");
    static const boost::regex pattern("^([^=]+)(=(.+))?$");
    boost::smatch match;
    if(boost::regex_search(attribute, match, pattern)) {
        std::string name = match[1].str();
        std::string value = match[3].str();
        // If there is no value then use the node's text
        if(value.length()==0) value = node.text();
        // If still no value then create an error
        if(value.length()==0){
            error_(node,"set-value-none",name,str(boost::format("No value provided for <%s>")%name));
            return "";
        }
        // Assign the variable in the new frame
        context->assign(name,value);
        return name;
    }
    else {
        error_(node,"set-syntax",attribute,str(boost::format("Syntax error in attribute <%s>")%attribute));
        return "";
    }
}

/**
 * Render a `par` element (e.g. `<span data-par="answer:number=42"></span>`)
 */
std::array<std::string,3> par_(Node node, Context* context,bool primary=true){
    std::string attribute = node.attr("data-par");
    static const boost::regex pattern("^([^:=]+)(:([a-z_]+))?(=(.+))?$");
    boost::smatch match;
    std::string name;
    std::string type;
    std::string default_;
    if(boost::regex_search(attribute, match, pattern)) {
        name = match[1].str();
        type = match[3].str();
        default_ = match[5].str();
        if(primary){
            Node input = node.select("input");
            if(not input) input = node.append("input");
            // Set name
            input.attr("name",name);
            // Set type
            if(type.length()>0) input.attr("type",type);
            // Get value, using default if not defined
            std::string value = input.attr("value");
            if(value.length()==0 and default_.length()>0){
                value = default_;
                input.attr("value",default_);
            }
            // Convert value into a valid expression
            if(value.length()>0){
                std::string expression;
                if(type=="text") expression = "\""+value+"\"";
                //..other HTML5 input types to be evaluated
                else expression = value;
                context->assign(name,expression);
            }
        }
    }
    else {
        error_(node,"par-syntax",attribute,str(boost::format("Syntax error in attribute <%s>")%attribute));
    }
    return {name,type,default_};
}

/**
 * Render a `text` element (e.g. `<span data-text="result"></span>`)
 *
 * The expression in the `data-text` attribute is converted to a 
 * character string by the context and used as the element's text.
 * If the element has a `data-off="true"` attribute then the element will not
 * be rendered and its text will remain unchanged.
 */
void text_(Node node, Context* context){
    if(node.attr("data-lock")!="true"){
        std::string expression = node.attr("data-text");
        std::string text = context->write(expression);
        node.text(text);
    }
}

/**
 * Render a `with` element (e.g. `<div data-with="sales"><span data-text="sum(quantity*price)" /></div>` )
 *
 * The expression in the `data-with` attribute is evaluated and made the subject of a new context frame.
 * All child nodes are rendered within the new frame. The frame is then exited.
 */
void with_(Node node, Context* context){
    std::string expression = node.attr("data-with");
    context->enter(expression);
    children_(node,context);
    context->exit();
} 

/**
 * Render a `if` element (e.g. `<div data-if="answer==42">...</div>` )
 *
 * The expression in the `data-if` attribute is evaluated in the context.
 */
void if_(Node node, Context* context){
    std::string expression = node.attr("data-if");
    bool hit = context->test(expression);
    if(hit){
        node.erase("data-off");
        children_(node,context);
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
                hit = context->test(expression);
                if(hit){
                    next.erase("data-off");
                    children_(next,context);
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
                children_(next,context);
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
void switch_(Node node, Context* context){
    std::string expression = node.attr("data-switch");
    context->mark(expression);

    bool matched = false;
    for(Node child : node.children()){
        if(child.has("data-case")){
            if(matched){
                child.attr("data-off","true");
            } else {
                std::string match = child.attr("data-case");
                matched = context->match(match);
                if(matched){
                    child.erase("data-off");
                    element_(child,context);
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
                element_(child,context);
            }
        } else {
            element_(child,context);
        }
    }

    context->unmark();
}

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
void for_(Node node, Context* context){
    std::string parts = node.attr("data-for");
    // Get the name of `item` and the `items` expression
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
    bool more = context->begin(item,items);
    // Get the first child element which will be repeated
    Node first = node.first_element();
    // If this for loop has been rendered before then the first element will have a `data-off`
    // attribute. So erase that attribute so that the repeated nodes don't get it
    if(first) first.erase("data-off");
    // Iterate
    int count = 0;
    while(first and more){
        // See if there is an existing child with a corresponding `data-index`
        std::string index = boost::lexical_cast<std::string>(count);
        // Must select only children (not other decendents) to prevent messing with
        // nested loops. 
        // Currently, our CSS selector implementation does not support this syntax:
        //     > [data-index="0"]
        // so use XPath instead:
        Node item = node.select("./*[@data-index='"+index+"']","xpath");
        if(item){
            // If there is, check to see if it is locked
            Node locked = item.select("./*[@data-lock]","xpath");
            if(not locked){
                // If it is not locked, then destroy and replace it
                item.destroy();
                item = node.append(first);
            }
        } else {
            // If there is not, create one
            item = node.append(first);
        }
        // Set index attribute
        item.attr("data-index",index);
        // Render the element
        element_(item,context);
        // Ask context to step to next item
        more = context->next();
        count++;
    }
    // Deactivate the first child
    if(first) first.attr("data-off","true");
    // Remove any children having a `data-index` attribute greater than the 
    // number of items, unless it has a `data-lock` decendent
    Nodes indexeds = node.filter("./*[@data-index]","xpath");
    for(Node indexed : indexeds){
        std::string index_string = indexed.attr("data-index");
        int index = boost::lexical_cast<int>(index_string);
        if(index>count-1){
            Node locked = indexed.select("[data-lock]");
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
void include_(Node node, Context* context){
    std::string include = node.attr("data-include");
    std::string version = node.attr("data-version");
    std::string select = node.attr("data-select");

    // If this node has been rendered before then there will be 
    // a `data-included` node. If it does not yet exist then append one.
    Node included = node.select("[data-included]");
    if(not included) included = node.append("div",{{"data-included","true"}});

    // If the included node has been edited then it may have a data-lock
    // element. If it does not have then clear and reinclude
    Node lock = included.select("[data-lock=\"true\"]");
    if(not lock) {
        // Clear the included node
        included.clear();
        //Obtain the included stencil...
        Node stencil;
        //Check to see if this is a "self" include, otherwise obtain the stencil
        if(include==".") stencil = node.root();
        else stencil = Component::get(include,version).as<Stencil>();
        // ...select from it
        if(select.length()>0){
            // ...append the selected nodes.
            for(Node node : stencil.filter(select)){
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
            for(Node modifier : node.filter("["+attribute+"]")){
                std::string selector = modifier.attr(attribute);
                for(Node target : included.filter(selector)){
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
    }

    // Enter a new namespace.
    // Do this regardless of whether there are any 
    // `data-par` elements, to avoid the included elements polluting the
    // main context or overwriting variables inadvertantly
    context->enter();

    // Apply `data-set` elements
    // Apply all the `set`s specified in the include first. This
    // may include setting vatiables not specified by the author of the included stencil.
    std::vector<std::string> assigned;
    for(Node set : node.filter("[data-set]")){
        std::string name = set_(set,context);
        assigned.push_back(name);
    }
    // Now apply the included element's parameters
    bool ok = true;
    for(Node par : included.filter("[data-par]")){
        auto parts = par_(par,context,false);
        std::string name = parts[0];
        std::string default_ = parts[2];
        // Check to see if it has already be assigned
        if(std::count(assigned.begin(),assigned.end(),name)==0){
            if(default_.length()>0){
                // Assign the default_ in the new frame
                context->assign(name,default_);
            } else {
                // Set an error
                error_(node,"par-required",name,str(boost::format("Parameter <%s> is required because it has no default")%name));
                ok  = false;
            }
        }
        // Remove the parameter, there is no need to have it in the included node
        par.destroy();
    }

    // Render the `data-included` element
    if(ok) children_(included,context);
    
    // Exit the included node
    context->exit();
}

void element_(Node node, Context* context){
    try {
        // Remove any existing errors
        for(Node child : node.filter("[data-error]")) child.destroy();
        // Check for handled elements
        // For each attribute in this node...
        //...use the name of the attribute to dispatch to another rendering method
        //   Note that return is used so that only the first Stencila "data-xxx" will be 
        //   considered and that directive will determine how/if children nodes are processed
        std::string tag = node.name();
        for(std::string attr : node.attrs()){
            // `macro` elements are not rendered
            if(attr=="data-macro") return ;
            else if(attr=="data-code") return code_(node,context);
            else if(attr=="data-set"){
                set_(node,context);
                return;
            }
            else if(attr=="data-par"){
                par_(node,context);
                return;
            }
            else if(attr=="data-text") return text_(node,context);
            else if(attr=="data-with") return with_(node,context);
            else if(attr=="data-if") return if_(node,context);
            // Ignore `elif` and `else` elements as these are processed by `if_`
            else if(attr=="data-elif" or attr=="data-else") return;
            else if(attr=="data-switch") return switch_(node,context);
            else if(attr=="data-for") return for_(node,context);
            else if(attr=="data-include") return include_(node,context);
        }
        // If return not yet hit then process children of this element
        children_(node,context);
    }
    catch(const std::exception& exc){
        error_(node,"exception","",exc.what());
    }
    catch(...){
        error_(node,"unknown","","Unknown exception");
    }
}

} // namespace


Stencil& Stencil::attach(Context* context){
    if(context_) delete context_;
    context_ = context;
    return *this;
}

Stencil& Stencil::detach(void){
    if(context_) delete context_;
    context_ = nullptr;
    return *this;
}

std::string Stencil::context(void) const {
    if(context_) return context_->details();
    else return "none";
}

Stencil& Stencil::render(Context* context){
    // If a different context, attach the new one
    if(context!=context_) attach(context);
    // Change to the stencil's directory
    boost::filesystem::path cwd = boost::filesystem::current_path();
    boost::filesystem::path path = boost::filesystem::path(Component::path(true));
    try {
        boost::filesystem::current_path(path);
    } catch(const std::exception& exc){
        STENCILA_THROW(Exception,str(boost::format("Error setting directory to <%s>")%path));
    }
    // Render root element within context
    element_(*this,context);
    // Return to the cwd
    boost::filesystem::current_path(cwd);
    return *this;
}

Stencil& Stencil::render(const std::string& type){
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
    // Render the stencil in the corresponding context type
    if(use=="py"){
        #if STENCILA_PYTHON_CONTEXT
            return render(new PythonContext);
        #else
            STENCILA_THROW(Exception,"Stencila has not been compiled with support for Python contexts");
        #endif
    }
    else if(use=="r"){
        #if STENCILA_R_CONTEXT
            return render(new RContext);
        #else
            STENCILA_THROW(Exception,"Stencila has not been compiled with support for R contexts");
        #endif
    }
    else {
       STENCILA_THROW(Exception,"Unrecognised context type: "+type); 
    }
    return *this;
}

Stencil& Stencil::render(void){
    if(context_) return render(context_);
    else return render(std::string());
    return *this;
}

} // namespace Stencila
