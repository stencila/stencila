#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/xpressive/xpressive_static.hpp>
#include <boost/xpressive/regex_compiler.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::cila(void) const {
    // Create a std::ostream and pass to cila(std::ostream)
    std::ostringstream stream;
    cila(stream);
    return stream.str();
}

const Stencil& Stencil::cila(std::ostream& stream) const {
    // Generate Cila with this Stencil as the root Node
    cila(*this,stream);
    return *this;
}

void Stencil::cila(Node node, std::ostream& stream, std::string indent) {
    // Generate Cila for a Xml::Node
    // For a document...
    if(node.is_document()){
        // .... generate Cila for each cild with no indentation
        for(Node child : node.children()) cila(child,stream);
    }
    // For an element...
    else if(node.is_element()){
        // Unless this is the very first content written to the stream
        // start elements on a new line with appropriate indentation
        if(stream.tellp()>0) stream<<"\n"<<indent;
        // Get element name and attributes
        std::string name = node.name();    
        auto attrs = node.attrs();
        // If this has no attributes...
        if(attrs.size()==0) {
            //... then output the node name (this needs to be done for <div>s too 
            // otherwise you get a blank line
            stream<<name;
        } else {
            // If this is not a <div> then output name
            bool div = true;
            if(name!="div"){
                stream<<name;
                div = false;
            }

            // id and class notations
            std::string id = node.attr("id");
            if(id.length()) stream<<"#"<<id;

            std::string class_ = node.attr("class");
            if(class_.length()) stream<<"."<<class_;

            // other attributes to go here in square braces

            // Status attributes
            std::string index = node.attr("data-index");
            if(index.length()) stream<<"@"<<index;

            std::string off = node.attr("data-off");
            if(off.length()) stream<<"-";    

            std::string lock = node.attr("data-lock");
            if(lock.length()) stream<<"!";    

            // Directive attributes. An element can only have one of these.
            // These need to go after the other attributes
            for(std::string attr : attrs){
                if(attr=="data-macro"){
                    if(not div) stream<<"!";
                    stream<<"macro";
                    break;
                }

                else if(attr=="data-code"){
                    if(not div) stream<<" ";
                    stream<<"code ";
                    break;
                }

                else if(attr=="data-text"){
                    if(not div) stream<<" ";
                    stream<<"text "<<node.attr("data-text");
                    break;
                }
                else if(attr=="data-image"){
                    if(not div) stream<<" ";
                    stream<<"image "<<node.attr("data-image");
                    break;
                }

                else if(attr=="data-with"){
                    if(not div) stream<<"!";
                    stream<<"with "<<node.attr("data-with");
                    break;
                }

                else if(attr=="data-if"){
                    if(not div) stream<<" ";
                    stream<<"if "<<node.attr("data-if");
                    break;
                }
                else if(attr=="data-elif"){
                    if(not div) stream<<" ";
                    stream<<"elif "<<node.attr("data-elif");
                    break;
                }
                else if(attr=="data-else"){
                    if(not div) stream<<" ";
                    stream<<"else";
                    break;
                }

                else if(attr=="data-switch"){
                    if(not div) stream<<" ";
                    stream<<"switch "<<node.attr("data-switch");
                    break;
                }
                else if(attr=="data-case"){
                    if(not div) stream<<" ";
                    stream<<"case "<<node.attr("data-case");
                }
                else if(attr=="data-default"){
                    if(not div) stream<<" ";
                    stream<<"default";
                }

                else if(attr=="data-for"){
                    if(not div) stream<<" ";
                    stream<<"for "<<node.attr("data-for");
                }
                else if(attr=="data-each"){
                    if(not div) stream<<" ";
                    stream<<"each "<<node.attr("data-each");
                }

                else if(attr=="data-include"){
                    if(not div) stream<<" ";
                    stream<<node.attr("data-include");
                }
            }
        }

        for(Node child : node.children()) cila(child,stream,indent+"\t");
    }
    else if(node.is_text()){
        std::string text = node.text();
        if(text.length()<100) stream<<" "<<text;
        else stream<<"\n"<<indent<<text;
    }
}

Stencil& Stencil::cila(const std::string& string){
    // Convert the std::string to a std::istream
    // and pass to cila(std::istream&)
    std::stringstream stream(string);
    stream.seekg(0);
    return cila(stream);
}

Stencil& Stencil::cila(std::istream& stream){
    // Define language grammar
    using namespace boost::xpressive;
    sregex indent = *space;
    /*
    List of vaild HTML5 element names from 
        http://www.w3.org/TR/html-markup/elements.html
    and extracted using this python script:
        import requests
        import bs4
        page = requests.get('http://www.w3.org/TR/html-markup/elements.html').text
        elems = bs4.BeautifulSoup(page).findAll('span', {'class':'element'})
        print '|'.join('"%s"'%elem.text for elem in sorted(set(elems)))
    */
    sregex tag = 
        #if 0 
        /*
        Statically compiling element name list dramatically increases compile
        times (e.g. 11s to 27s) and executable sizes (e.g. 10Mb to 80Mb).
        */
        as_xpr("a")|"abbr"|"address"|"area"|"article"|"aside"|"audio"|"b"|"base"|"bdi"|"bdo"|"blockquote"|"body"|"br"|"button"|
            "canvas"|"caption"|"cite"|"code"|"col"|"colgroup"|"command"|"datalist"|"dd"|"del"|"details"|"dfn"|"div"|"dl"|"dt"|
            "em"|"embed"|"fieldset"|"figcaption"|"figure"|"footer"|"form"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"head"|"header"|"hgroup"|"hr"|"html"|
            "i"|"iframe"|"img"|"input"|"ins"|"kbd"|"keygen"|"label"|"legend"|"li"|"link"|"map"|"mark"|"menu"|"meta"|"meter"|"nav"|"noscript"|
            "object"|"ol"|"optgroup"|"option"|"output"|"p"|"param"|"pre"|"progress"|"q"|"rp"|"rt"|"ruby"|"s"|"samp"|"script"|"section"|
            "select"|"small"|"source"|"span"|"strong"|"style"|"sub"|"summary"|"sup"|"table"|"tbody"|"td"|"textarea"|"tfoot"|"th"|"thead"|
            "time"|"title"|"tr"|"track"|"u"|"ul"|"var"|"video"|"wbr"
        #else
        /*
        Dynamically compiling element name list only slightly increases compile
        times (e.g. 11s to 15s) and executable sizes (e.g. 10Mb to 13Mb).
        */
        sregex::compile(
            "a|abbr|address|area|article|aside|audio|b|base|bdi|bdo|blockquote|body|br|button|"
            "canvas|caption|cite|code|col|colgroup|command|datalist|dd|del|details|dfn|div|dl|dt|"
            "em|embed|fieldset|figcaption|figure|footer|form|h1|h2|h3|h4|h5|h6|head|header|hgroup|hr|html|"
            "i|iframe|img|input|ins|kbd|keygen|label|legend|li|link|map|mark|menu|meta|meter|nav|noscript|"
            "object|ol|optgroup|option|output|p|param|pre|progress|q|rp|rt|ruby|s|samp|script|section|"
            "select|small|source|span|strong|style|sub|summary|sup|table|tbody|td|textarea|tfoot|th|thead|"
            "time|title|tr|track|u|ul|var|video|wbr"
        )
        #endif
    ;

    sregex inlined_expr      = *(~(set='|'));
    sregex inlined           = *tag >> "|" >> inlined_expr >> "|";
    sregex chars            = *space>>+(~(set='|'))>>*space;
    sregex text             = +(inlined|chars);

    sregex expr = +_;
    sregex selector = +_;
    sregex address = +_w;

    sregex directive_for = as_xpr("for") >> +space >> expr >> +space >> "in" >> +space >> expr;
    sregex directive_include = as_xpr("include") >> +space >> address >> *(+space >> selector);
    sregex directive_modifier_name = sregex::compile("replace|before|after|prepend|append");
    sregex directive_modifier = directive_modifier_name >> +space >> selector;
    sregex directive_arg_name = sregex::compile("text|with|if|elif|switch|value");
    sregex directive_arg = directive_arg_name >> +space >> expr;
    sregex directive_noarg = sregex::compile("script|else|default");

    sregex attr_identifier  = +(_w|'-');
    sregex attr_string      = ('\"' >> *(~(set='\r','\n','\"')) >> '\"') | 
                               ('\'' >> *(~(set='\r','\n','\'')) >> '\'');
    sregex attr_class       = '.' >> attr_identifier;
    sregex attr_id          = as_xpr("#") >> attr_identifier;
    sregex attr_assign      = attr_identifier >> '=' >> attr_string;

    sregex element          = (
        (*(tag >> "!") >> (directive_include|directive_modifier|directive_for|directive_arg|directive_noarg)) |
        (tag >> *(attr_class|attr_id|'[' >> *space >> +(attr_assign>>*space) >> ']')) |
        +(attr_class|attr_id|'[' >> *space >> +(attr_assign>>*space) >> ']')
    ) >> *(+space >> *chars);

    sregex code = sregex::compile("py|r");

    sregex equation_text = *(~(set='`')); 
    sregex equation = as_xpr("`") >> equation_text >> as_xpr("`");

    sregex comment_text = *_;
    sregex comment = as_xpr("//") >> comment_text;

    sregex regex = indent >> (comment|equation|code|element|text);

    // Clear the stencil of all existing content
    clear();
    // Define modes for parsing
    enum {normal_mode,code_mode} mode = normal_mode;
    // Define a parent node, starting off as the stencil's
    // content root node
    Node parent = *this;
    // Define current node that may become parent
    Node current;
    // Define a stack of <indent,Node> pairs
    // when the indentation increases then then `currrent`
    // becomes parent
    std::deque<std::pair<int,Node>> levels;
    levels.push_back({0,*this});
    // Define code which is appended
    std::string code_content;
    std::string code_lang;
    // Keep track if previous line was empty
    bool previous_empty = true;
    // Iterate over lines
    std::string line;
    uint count = 0;
    while(std::getline(stream,line,'\n')){
        // Increment the line counter
        count++;
        // Determine if this line is empty - anything other than whitespace?
        bool empty = line.find_first_not_of("\t ")==std::string::npos;
        // Determine the parent-child relationships for this node based on its indentation
        if(not empty){
            int indentation = line.find_first_not_of("\t");
            // If indentation has increased, the current node becomes the parent
            int last = levels.back().first;
            if(indentation>last){
                levels.push_back({indentation,current});
                parent = current;
            }
            // if it has not changed, then do nothing
            else if(indentation==last){
            }
            // if it less, then pop off parents until we get to the right level
            else {
                while(levels.size()>1 and indentation<levels.back().first) levels.pop_back();
                parent = levels.back().second;
            }
        }
        if(mode==normal_mode){
            // Trim the line of extraneous whitespace
            boost::trim(line);
            // If the line has content...
            if(not empty){
                // Parse the line into a syntax tree
                smatch tree;
                regex_match(line,tree,regex);
                // The line has several branches; get the branches iterator
                auto branch = tree.nested_results().begin();
                // Skip the first branch for the `indent`
                branch++;
                // Get the id of the next branch
                const void* id = branch->regex_id();
                if(id==element.regex_id()){
                    // Get iterator for subtree
                    auto tree = *branch;
                    auto branch = tree.nested_results().begin();
                    // The first branch is always a tag or an attr
                    // If it is an tag use that, otherwise make it a <div>
                    std::string name = (branch->regex_id()==tag.regex_id())?branch->str():"div";
                    // Create the element
                    current = parent.append(name);
                    // Iterate over remaining branches which include attributes for the element
                    // including those for stencil directives.
                    // Note that since the first branch may need further processing (if it is an attribute)
                    // that the branch iterator is not incremented until the end of the loop.
                    while(branch!=tree.nested_results().end()){
                        const void* id = branch->regex_id();

                        if(id==directive_include.regex_id()) {
                            auto attr = branch->nested_results().begin();
                            current.attr("data-include",attr->str());
                            //if(branch->nested_results().size()>1){
                            //    auto selector = ++nested;
                            //    current.attr("data-select",selector->str());
                            //}
                        }
                        else if(id==directive_for.regex_id()) {
                            // Get item and items
                            auto nested = branch->nested_results().begin();
                            auto item = nested;
                            auto items = ++nested;
                            // Set for attribute
                            current.attr("data-for",item->str()+":"+items->str());
                        }
                        else if(id==directive_arg.regex_id() or id==directive_modifier.regex_id()) {
                            // Get name and value
                            auto nested = branch->nested_results().begin();
                            auto name = nested;
                            auto value = ++nested;
                            // Set directive attribute
                            current.attr("data-"+name->str(),value->str());
                        }
                        else if(id==directive_noarg.regex_id()){
                            // Set empty directive attribute
                            current.attr("data-"+branch->str(),"");
                        }
                        else if(id==attr_id.regex_id()) {
                            // Set `id` after removing leading "#"
                            current.attr("id",branch->str(0).erase(0,1));
                        }
                        else if(id==attr_class.regex_id()){
                            // Set `class` after removing leading "."
                            current.attr("class",branch->str(0).erase(0,1));
                        }
                        else if(id==attr_assign.regex_id()){
                            // Get name and value
                            auto nested = branch->nested_results().begin();
                            auto name = nested;
                            auto value = ++nested;
                            // Remove leading and trailing quotes from value
                            std::string string = value->str();
                            string.erase(0,1);
                            string.erase(string.length()-1,1);
                            // Set the attribute
                            current.attr(name->str(),string);
                        }
                        else if(id==chars.regex_id()){
                            current.append_text(branch->str());
                        }
                        branch++;
                    }
                }
                else if(id==text.regex_id()){
                    // If previous line was empty then create a new paragraph to be target
                    // for additional text, otherwise use existing parent as target
                    Node target;
                    if(previous_empty){
                        current = parent.append("p");
                        target = current;
                    } else {
                        target = parent;
                    }
                    // `text` nodes are made up of one or more `chars` and `inlined` `data-text` directives
                    auto tree = *branch;
                    for(auto branch = tree.nested_results().begin();branch!=tree.nested_results().end();branch++){
                        const void* id = branch->regex_id();
                        if(id==chars.regex_id()){
                            target.append_text(branch->str());
                        }
                        else if(id==inlined.regex_id()) {
                            /*
                            std::string element_name = "span";
                            auto expression = branch;
                            if(tree.nested_results().size()==2){
                                element_name = branch->str();
                                expression = ++branch;
                            }
                            Xml::Node self = Xml::Document::append(node,element_name);
                            Xml::Document::set(self,"data-text",expression->str());
                            */
                        }
                    }
                }
                else if(id==code.regex_id()){
                    // Append the node
                    current = parent.append("pre");
                    // Reset code content
                    // A starting newline is required to escape the commented "<![CDATA[" line
                    code_content = "\n";
                    // The code language is the the branch string
                    code_lang = branch->str();
                    current.attr("data-code",code_lang);
                    // Turn on code mode
                    mode = code_mode;
                }
                else if(id==equation.regex_id()){
                    // Create a <p class="equation">
                    current = parent.append("p",{{"class","equation"}});
                    // Get equation_text
                    std::string equation_text;
                    auto tree = *branch;
                    auto text = tree.nested_results().begin();
                    if(text != tree.nested_results().end()) equation_text = text->str();
                    // Append equation text to current node with surrounding backticks
                    current.append_text("`"+equation_text+"`");
                }
                else if(id==comment.regex_id()){
                }
                else {
                    STENCILA_THROW(Exception,"Unrecognised syntax: "+boost::lexical_cast<std::string>(count)+": "+line);
                }
            }
        }
        else if(mode==code_mode){
            if(empty){
                // Add to code
                code_content += line + "\n";
            } else {
                // Add a comment token to escape "<![CDATA[" . This needs to be added to `current` NOT `code_content`
                if(code_lang=="r" or code_lang=="py") current.append_text("#");
                // Add a comment token to escape "]]>". This needs to be added to `code_content` before appending as CDATA
                if(code_lang=="r" or code_lang=="py") code_content += "#";
                // Then add the code as CDATA
                current.append_cdata(code_content);
                // Turn off code mode
                mode = normal_mode;
            }
        }

        previous_empty = empty;
    }
    return *this;
}

}
