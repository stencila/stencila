#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

#include <iostream>

namespace Stencila {

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

void Stencil::render_error(Node node, const std::string& type, const std::string& data, const std::string& message){
    node.attr("data-error",type+"~"+data+"~"+message);
}

void Stencil::render_exec(Node node, Context* context){
    // Check if this `code` directive needs to be executed
    if(not render_hash(node)) return;
    // Get the list of contexts and ensure this context is in the list
    std::string contexts = node.attr("data-exec");
    std::vector<std::string> items = split(contexts,",");
    bool ok = false;
    for(std::string& item : items){
        trim(item);
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
            // Default images sizes and units based on the width of an A4 page having
            // 2cm margins.
            if(width=="") width = "17";
            if(height=="") height = "17";
            if(units=="") units = "cm";
            // Execute
            std::string output = context->execute(code,hash_,format,width,height,units);
            // Remove any existing output
            Node next = node.next_element();
            if(next and next.attr("data-out")=="true") next.destroy();
            // Append new output
            if(format.length()){
                Xml::Document doc;
                Node output_node;
                if(format=="text"){
                    output_node = doc.append("samp",output);
                }
                else if(format=="png" or format=="svg"){
                    output_node = doc.append("img",{
                        {"src",output}
                    });
                }
                else {
                    render_error(node,"out-format",format,"Output format not recognised: "+format);
                }
                if(output_node){
                    // Flag output node 
                    output_node.attr("data-out","true");
                    // Create a copy immeadiately after code directive
                    node.after(output_node);
                }
            }
        }
    }
}

std::string Stencil::render_set(Node node, Context* context){
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
            render_error(node,"set-value-none",name,"No value provided for <"+name+">");
            return "";
        }
        // Assign the variable in the new frame
        context->assign(name,value);
        return name;
    }
    else {
        render_error(node,"set-syntax",attribute,"Syntax error in attribute <"+attribute+">");
        return "";
    }
}

void Stencil::render_par(Node node, Context* context){
    Parameter par(node);
    if(par.ok){
        auto name = par.name;
        auto type = par.type;
        auto default_ = par.default_;
        Node input = node.select("input");
        if(not input) input = node.append("input");
        // Set name
        input.attr("name",name);
        // Set type
        if(type.length()) input.attr("type",type);
        // Get value, using default if not defined
        std::string value = input.attr("value");
        if(not value.length() and par.default_.length()){
            value = default_;
            input.attr("value",value);
        }
        // Set value in the context
        if(value.length()>0){
            context->input(name,type,value);
        }
        // Render input node
        render_input(input,context);
    }
    else {
        render_error(node,"par-syntax",par.attribute,"Syntax error in attribute <"+par.attribute+">");
    }
}

void Stencil::render_write(Node node, Context* context){
    if(node.attr("data-lock")!="true"){
        std::string expression = node.attr("data-write");
        std::string text = context->write(expression);
        node.text(text);
    }
}

void Stencil::render_with(Node node, Context* context){
    std::string expression = node.attr("data-with");
    context->enter(expression);
    render_children(node,context);
    context->exit();
} 

void Stencil::render_if(Node node, Context* context){
    std::string expression = node.attr("data-if");
    bool hit = context->test(expression);
    if(hit){
        node.erase("data-off");
        render_children(node,context);
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
                    render_children(next,context);
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
                render_children(next,context);
            }
            break;
        }
        else break;
        next = next.next_element();
    }
}

void Stencil::render_switch(Node node, Context* context){
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
                    render(child,context);
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
                render(child,context);
            }
        } else {
            render(child,context);
        }
    }

    context->unmark();
}

void Stencil::render_for(Node node, Context* context){
    std::string attribute = node.attr("data-for");
    // Get the name of `item` and the `items` expression
    std::string name;
    std::string expr;
    static const boost::regex pattern("^(\\w+)\\s+in\\s+(.+)$");
    boost::smatch match;
    if(boost::regex_search(attribute, match, pattern)) {
        name = match[1].str();
        expr = match[2].str();
    }
    else {
        STENCILA_THROW(Exception,"Syntax error in for directive attribute <"+attribute+">");
    }
    // Initialise the loop
    bool more = context->begin(name,expr);
    // Get the first child element which will be repeated
    Node first = node.first_element();
    // If this for loop has been rendered before then the first element will have a `data-off`
    // attribute. So erase that attribute so that the repeated nodes don't get it
    if(first) first.erase("data-off");
    // Iterate
    int count = 0;
    while(first and more){
        // See if there is an existing child with a corresponding `data-index`
        std::string index = string(count);
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
        render(item,context);
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
        int index = unstring<int>(index_string);
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

void Stencil::render_include(Node node, Context* context){
    std::string include_expr = node.attr("data-include");
    std::string version = node.attr("data-version");
    std::string select = node.attr("data-select");

    // Obtain string representation of include_expr
    std::string include = include_expr;
    if(include_expr!=".") context->write(include_expr);

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
        Node includee;
        //Check to see if this is a "self" include, otherwise obtain the includee
        if(include==".") includee = node.root();
        else includee = Component::get(include,version).as<Stencil>();
        // ...select from it
        if(select.length()>0){
            // ...append the selected nodes.
            for(Node node : includee.filter(select)){
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
            // ...append the entire includee. 
            // No attempt is made to remove macros when included an entire includee.
            // Must add each child because includee is a document (see `Node::append(const Document& doc)`)
            for(auto child : includee.children()) included.append(child);
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
    // may include setting variables not specified as parameters
    // by the author of the included stencil.
    std::vector<std::string> assigned;
    for(Node set : node.filter("[data-set]")){
        std::string name = render_set(set,context);
        assigned.push_back(name);
    }
    // Now apply the included element's parameters
    bool ok = true;
    for(Node par : included.filter("[data-par]")){
        Parameter parameter(par);
        if(parameter.ok){
            auto name = parameter.name;
            auto type = parameter.type;
            auto default_ = parameter.default_;
            // Check to see if it has already be assigned
            if(std::count(assigned.begin(),assigned.end(),name)==0){
                if(default_.length()){
                    // Assign the default_ in the new frame
                    context->assign(name,default_);
                } else {
                    // Set an error
                    render_error(node,"par-required",name,"Parameter <"+name+"> is required because it has no default");
                    ok  = false;
                }
            }
        }
        // Remove the parameter, there is no need to have it in the included node
        par.destroy();
    }

    // Render the `data-included` element
    if(ok) render_children(included,context);
    
    // Exit the included node
    context->exit();
}

void Stencil::render_input(Node node, Context* context){
    if(render_hash(node)){
        auto name = node.attr("name");
        auto type = node.attr("type");
        auto value = node.attr("value");
        context->input(name,type,value);
    }
}

void Stencil::render_children(Node node, Context* context){
    for(Node child : node.children()) render(child,context);
}

struct Stencil::Outline {

    struct Level {

        Level* parent = nullptr;
        int level = 0;
        int index;
        std::string label;
        std::string id;
        std::vector<Level*> sublevels;

        ~Level(void){
            for(auto* level : sublevels) delete level;
        }

        Level* sublevel(void){
            Level* sublevel = new Level;
            sublevel->level = level+1;
            sublevel->index = sublevels.size()+1;
            sublevel->parent = this;
            sublevels.push_back(sublevel);
            return sublevel;
        }

        std::string path(const std::string& sep=".") const {
            std::string path;
            const Level* next = this;
            while(next->index){
                if(path.length()) path.insert(0,sep);
                path.insert(0,string(next->index));
                next = next->parent;
            }
            return path;
        }

        std::string id_(void) const {
            return "section-"+path("-");
        }

        std::string class_(void) const {
            return "level-"+string(level);
        }

        void heading(Node node){
            if(label.length()==0){
                // Get label for this level from the node
                label = node.text();
                // Check for node id, create one if needed, then add it to 
                // level for links and to the section header
                auto id_value = node.attr("id");
                if(not id_value.length()){
                    id_value = id_();
                    node.attr("id",id_value);
                }
                id = id_value;
                // Check for an existing label
                std::string path_string = path();
                Node label = node.select(".label");
                if(not label){
                    // Prepend a label
                    label = node.prepend("span");
                    label.attr("class","label");
                    label.append("span",{{"class","path"}},path_string);
                    label.append("span",{{"class","separator"}}," ");
                } else {
                    // Ammend the label
                    Node path = label.select(".path");
                    if(not path) path = label.append("span",{{"class","path"}},path_string);
                    else path.text(path_string);
                }            
                // Give class to the heading for styling
                node.attr("class",class_());
            }
        }

        void render(Node ul) const {  
            Node li = ul.append(
                "li",
                {{"class",class_()}}
            );
            li.append(
                "a",
                {{"href","#"+id}},
                path()+" "+label
            );
            for(auto* level : sublevels) level->render(ul);
        }
    };

    Level* root;
    Level* current;
    Node node;

    Outline(void){
        root = new Level;
        current = root;
    }

    ~Outline(void){
        if(root) delete root;
    }

    void enter(void){
        current = current->sublevel();
    }

    void exit(void){
        current = current->parent;
    }

    void heading(Node node){
        current->heading(node);
    }

    void render(void){
        if(node) {
            Node ul = node.append("ul");
            root->render(ul);
        }
    }
};

void Stencil::render(Node node, Context* context){
    try {
        // Remove any existing error attribute
        node.erase("[data-error]");
        // Check for handled elements
        std::string tag = node.name();
        // For each attribute in this node...
        //...use the name of the attribute to dispatch to another rendering method
        //   Note that return is used so that only the first Stencila "data-xxx" will be 
        //   considered and that directive will determine how/if children nodes are processed
        for(std::string attr : node.attrs()){
            // `macro` elements are not rendered
            if(attr=="data-macro") return ;
            else if(attr=="data-exec") return render_exec(node,context);
            else if(attr=="data-set"){
                render_set(node,context);
                return;
            }
            else if(attr=="data-par") return render_par(node,context);
            else if(attr=="data-write") return render_write(node,context);
            else if(attr=="data-with") return render_with(node,context);
            else if(attr=="data-if") return render_if(node,context);
            // Ignore `elif` and `else` elements as these are processed by `if_`
            else if(attr=="data-elif" or attr=="data-else") return;
            else if(attr=="data-switch") return render_switch(node,context);
            else if(attr=="data-for") return render_for(node,context);
            else if(attr=="data-include") return render_include(node,context);
        }
        // Render input elements
        if(tag=="input"){
            counts_["input"]++;
            return render_input(node,context);
        }
        // Handle outline
        else if(node.attr("id")=="outline"){
            outline_->node = node;
        }
        // Handle sections
        else if(tag=="section"){
            // Enter a sublevel
            outline_->enter();
            // Render children
            render_children(node,context);
            // Exit sublevel
            outline_->exit();
            // Return so the render_children below is not hit
            return;
        }
        // Handle headings
        else if(tag=="h1"){
            outline_->heading(node);
        }
        // Handle table and figure captions
        else if(tag=="table" or tag=="figure"){
            Node caption = node.select("caption,figcaption");
            if(caption){
                // Increment the count for his caption type
                unsigned int& count = counts_[tag+" caption"];
                count++;
                std::string count_string = string(count);
                // Check for an existing label
                Node label = caption.select(".label");
                if(not label){
                    // Prepend a label
                    label = caption.prepend("span");
                    label.attr("class","label");
                    label.append("span",{{"class","type"}},tag=="table"?"Table":"Figure");
                    label.append("span",{{"class","number"}},count_string);
                    label.append("span",{{"class","separator"}},":");
                } else {
                    // Amend the label
                    Node number = label.select(".number");
                    if(not number) number = label.append("span",{{"class","number"}},count_string);
                    else number.text(count_string);
                }
                // Check for id - on table or figure NOT caption!
                std::string id = node.attr("id");
                if(not id.length()){
                    node.attr("id",tag+"-"+count_string);
                }
            }
        }
        // If return not yet hit then process children of this element
        render_children(node,context);
    }
    catch(const std::exception& exc){
        render_error(node,"exception","",exc.what());
    }
    catch(...){
        render_error(node,"unknown","","Unknown exception");
    }
}

bool Stencil::render_hash(Node node){
    // Create a key string for this node which starts with the current value
    // for the current cumulative hash and its attributes and text
    std::string key = hash_;
    for(auto attr : node.attrs()){
        if(attr!="data-hash") key += attr+":"+node.attr(attr);
    } 
    key += node.text();
    // Create a new integer hash
    static std::hash<std::string> hasher;
    std::size_t number = hasher(key);
    // To reduce its lenght, convert the integer hash to a 
    // shorter string by encoding using a character set
    static char chars[] = {
        'a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z',
        'A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
        '0','1','2','3','4','5','6','7','8','9'
    };
    std::string hash;
    while(number>0){
        int index = number % sizeof(chars);
        hash = chars[index] + hash;
        number = int(number/sizeof(chars));
    }
    // If this is a non-`const` node (not declared const) then update the cumulative hash
    // so that changes in this node cascade to other nodes
    if(node.attr("data-const")!="true") hash_ = hash;
    // If there is no change in the hash then return false
    // otherwise replace the hash (may be missing) and return true
    std::string current = node.attr("data-hash");
    if(hash==current) return false;
    else {
        node.attr("data-hash",hash);
        return true;
    }
}

void Stencil::render_initialise(Node node, Context* context){
    hash_ = "";

    if(outline_) delete outline_;
    outline_ = new Outline;
}

void Stencil::render_finalise(Node node, Context* context){
    outline_->render();

    // Render references
    for(Node ref : filter("[data-ref]")){
        ref.clear();
        std::string selector = ref.attr("data-ref");
        Node target = select(selector);
        Node label = target.select(".label");
        if(label){
            Node a = ref.append(
                "a",
                {{"href","#"+target.attr("id")}},
                label.select(".type").text() + " " + label.select(".number").text()
            );
        }
    }
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
        STENCILA_THROW(Exception,"Error setting directory to <"+path.string()+">");
    }
    // Reset flags and counts
    counts_["input"] = 0;
    counts_["table caption"] = 0;
    counts_["figure caption"] = 0;
    // Initlise rendering
    render_initialise(*this,context);
    // Render root element within context
    render(*this,context);
    // Finalise rendering
    render_finalise(*this,context);
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
}

Stencil& Stencil::restart(void){
    return strip().render();
}

Stencil& Stencil::strip(void){
    // Remove attributes added by `render()`
    for(std::string attr : {"data-hash","data-off","data-error"}){
        for(Node node : filter("["+attr+"]")) node.erase(attr);
    }
    // Remove elements added by `render()`
    for(Node node : filter("[data-index],[data-out],[data-included]")){
        node.destroy();
    }
    return *this;
}    

} // namespace Stencila
