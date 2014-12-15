#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/xpressive/xpressive_static.hpp>
#include <boost/xpressive/regex_compiler.hpp>
#include <boost/regex.hpp>

#include <stencila/stencil.hpp>

// Anonymous local namespace for Cila parsing and generating functions
// which do not need to be Stencil methods
namespace {

using namespace Stencila;
typedef Stencil::Node Node;

// Forward declarations of the main parsing and generating functions
// which are used recursively below
void parse(Node node, std::istream& stream);
void generate(Node node, std::ostream& stream, std::string indent);

// Enumeration for the Cila parsing mode
enum Mode {
    normal_mode,
    code_mode
};

// Parsing state information passed between parsing functions
struct State {
    Mode mode = normal_mode;

    bool end = false;

    struct Line {
        // Is the line blank? (ie. no non-whitespace characters)
        bool blank = false;
        unsigned int indentation = 0;
    };
    Line current;
    Line previous;

    struct {
        std::string lang;
        std::string content;
        unsigned int indentation = 0;
    } code;
};

void code_mode_start(const std::string& lang, State& state){
    // Set state variables (note indentation to one plus current) 
    state.mode = code_mode;
    state.code.lang = lang;
    state.code.content = "";
    state.code.indentation = state.current.indentation + 1;
}

void code_mode_check(Node parent, const std::string& line, State& state){
    // If line is blank or indented more...
    if(not state.end and (state.current.blank or state.current.indentation>=state.code.indentation)){
        // Add line, even if it is blank, to `code.content` but strip `code.indentation` from it first
        std::string stripped = line;
        if(stripped.length()>=state.code.indentation) stripped = stripped.substr(state.code.indentation);
        state.code.content += stripped + "\n";
    }
    // Otherwise finalise the code element
    else {
        std::string code = state.code.content;
        // Force starting and ending newlines
        // for aesthetics
        if(code[0]!='\n') code.insert(0,"\n");
        if(code[code.length()-1]!='\n') code += "\n";
        // Then add the code as plain text
        parent.append_text(code);
        // Turn off code mode
        state.mode = normal_mode;
    }
}

// Language grammar is defined below.
// For clarity, parsing and generating functions
// are defined adjacent to each element of the grammar
using namespace boost::xpressive;

/**
 * Markdown-like shortcuts for inline HTML elements
 *
 * These mimic Markdown behaviour by converting certain syntax into HTML elements.
 * These are parsed out of text within the `text_parse` function.
 */

/**
 * Inline code
 *
 * Called `mono` to prevent clashes with the 
 * special `code` directive below (used for executed code)
 */
mark_tag mono_content(1);
sregex mono = '`' >> (mono_content=-+_) >> '`';

void mono_parse(Node node, const smatch& tree){
    node.append("code",tree[mono_content].str());
}

void mono_gen(Node node, std::ostream& stream){
    stream<<'`'<<node.text()<<'`';
}

/**
 * Inline math 
 */
mark_tag math_content(1);
sregex math = '|' >> (math_content=-+_) >> '|';

void math_parse(Node node, const smatch& tree){
    Node span = node.append("span",{{"class","math"}});
    span.append("script",{{"type","math/asciimath"}},tree[math_content].str());
}

void math_gen(Node node, std::ostream& stream){
    Node script = node.select("script");
    stream<<'|'<<script.text()<<'|';
}

/**
 * Emphasis <em>
 */
mark_tag emphasis_content(1);
sregex emphasis = ('_' >> (emphasis_content=-+_) >> '_') |
                  ('*' >> (emphasis_content=-+_) >> '*');

void emphasis_parse(Node node, const smatch& tree){
    node.append("em",tree[emphasis_content].str());
}

void emphasis_gen(Node node, std::ostream& stream){
    stream<<'_'<<node.text()<<'_';
}

/**
 * Strong <strong>
 */
mark_tag strong_content(1);
sregex strong = (as_xpr("__") >> (strong_content=-+_) >> "__") |
                (as_xpr("**") >> (strong_content=-+_) >> "**");

void strong_parse(Node node, const smatch& tree){
    node.append("strong",tree[strong_content].str());
}

void strong_gen(Node node, std::ostream& stream){
    stream<<"__"<<node.text()<<"__";
}

/**
 * Hyperlinks
 */
mark_tag link_content(1);
mark_tag link_url(2);
sregex link = '[' >> (link_content=-+_) >> ']' >> '(' >> (link_url=-+_) >> ')';

void link_parse(Node node, const smatch& tree){
    node.append("a",{{"href",tree[link_url].str()}},tree[link_content].str());
}

void link_gen(Node node, std::ostream& stream){
    stream<<"["<<node.text()<<"]("<<node.attr("href")<<")";
}

sregex inlines = +(mono|math|strong|emphasis|link);

/**
 * Text
 */
sregex text = +_;

Node text_parse(Node parent, const smatch& tree, const State& state){
    // If previous line was blank then create a new paragraph to be target
    // for additional text, otherwise use existing parent as target
    Node node;
    if(state.previous.blank){
        node = parent.append("p");
    } else {
        node = parent;
    }
    // Text nodes may have nested lines defined using curly braces e.g.
    //   The minimum is {if a<b {text a} else {text b}}.
    // Deal with those by replacing { with indented lines and } with outdented lines
    // There is some escaping in the code below so that braces within `mono` and `math`
    // are ignored as are \{ and \}. However, this is probably not robust and we should consider
    // reimplementing this using regexes.
    std::string content = tree.str();
    bool nested = false;
    std::string formatted;
    formatted.reserve(content.length());
    std::deque<char> indent = {'\n'};
    char previous = 0;
    char protector = 0;
    for(std::string::iterator iter=content.begin();iter!=content.end();iter++){
        char current = *iter;
        if(current=='{' and previous!='\\' and protector==0){
            nested = true;
            // Add a newline with indentation if there is already some content
            if(formatted.length()>0) formatted.append(indent.begin(),indent.end());
            indent.push_back('\t');
        }
        else if(current=='}' and previous!='\\' and protector==0){
            formatted.push_back('\n');
            indent.pop_back();
        }
        else if(current=='`'){
            if(protector=='`') protector = 0;
            else protector = '`';
        }
        else if(current=='|'){
            if(protector=='|') protector = 0;
            else protector = '|';
        }
        else {
            formatted += current;
        }
    }
    // Remove any trailing newline
    if(formatted.back()=='\n') formatted.pop_back();

    if(not nested){
        std::string text = tree.str();
        // Search for inlines within in the text
        boost::xpressive::sregex_iterator iter(text.begin(), text.end(), inlines), end;
        // Iterate over any inlines, appending text in between them
        unsigned int last = 0;
        for (; iter != end; ++iter){
            // Get start and finish of inline
            auto submatch = (*iter)[0];
            unsigned int start = submatch.first - text.begin();
            unsigned int finish = start + submatch.length() - 1;
            // If there is any preceding text append it
            if(start>last){
                node.append_text(text.substr(last,start-last));
            }
            last = finish + 1;
            // The first, and only, nested result is the inline.
            // Get it and resolve it's id.
            auto result = iter->nested_results().begin();
            const void* id = result->regex_id();
            if(id==mono.regex_id()) mono_parse(node,*result);
            else if(id==math.regex_id()) math_parse(node,*result);
            else if(id==emphasis.regex_id()) emphasis_parse(node,*result);
            else if(id==strong.regex_id()) strong_parse(node,*result);
            else if(id==link.regex_id()) link_parse(node,*result);
            
        }
        // Append any trailing text
        if(last<text.length()) node.append_text(text.substr(last));
        
    }
    else {
        std::stringstream stream(formatted);
        stream.seekg(0);
        parse(node,stream);
    }
    return node;
}

/**
 * Markdown-like shortcuts for HTML elements
 *
 * These mimic Markdown behaviour by converting certain syntax into HTML elements.
 * See the `root` regex and the `parse` function for their application.
 */

mark_tag header_level(1);
mark_tag header_content(2);
sregex header = (header_level=repeat<1,6>('#')) >> +space >> (header_content=+_);

Node header_parse(Node parent, const smatch& tree, const State& state){
    unsigned int level = tree[header_level].str().length();
    return parent.append(
        "h"+boost::lexical_cast<std::string>(level),
        tree[header_content].str()
    );
}

mark_tag ul_content(1);
sregex ul = (set='*','-','+') >> +space >> (ul_content=+_);

Node ul_parse(Node parent, const smatch& tree, const State& state){
    return parent.append("li",tree[ul_content].str());
}

mark_tag ol_content(1);
sregex ol = +_d >> '.' >> +space >> (ol_content=+_);

Node ol_parse(Node parent, const smatch& tree, const State& state){
    return parent.append("li",tree[ol_content].str());
}

/**
 * HTML5 tags
 */
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
        "i|iframe|img|input|ins|kbd|keygen|label|legend|li|link|main|map|mark|menu|meta|meter|nav|noscript|"
        "object|ol|optgroup|option|output|p|param|pre|progress|q|rp|rt|ruby|s|samp|script|section|"
        "select|small|source|span|strong|style|sub|summary|sup|table|tbody|td|textarea|tfoot|th|thead|"
        "time|title|tr|track|u|ul|var|video|wbr"
    )
    #endif
;

/**
 * Attributes of elements
 */

sregex identifier  = +(_w|'-');

/**
 * A convienience function for generating attribute Cila
 */
void attr_gen(std::ostream& line, const std::string& string){
    if(line.tellp()>0) line<<" ";
    line<<string;
}

// id="bar"

sregex id = '#' >> identifier;

void id_parse(Node node, const smatch& tree){
    // Set `id` after removing leading "#"
    // This will overide any previous id assignments for this element
    node.attr("id",tree.str(0).erase(0,1));
}

void id_gen(Node node, std::ostream& line){
    auto id = node.attr("id");
    if(id.length()){
        // Id is reduntant if this is a macro so do not output id
        // if this node is a macro
        auto macro = node.attr("data-macro");
        if(macro.length()==0) attr_gen(line,"#"+id);
    };
}

// class="foo"

sregex class_ = '.' >> identifier;

void class_parse(Node node, const smatch& tree){
    // Concatenate to `class` after removing leading "."
    // This will "accumulate" any class assignments
    node.concat("class",tree.str(0).erase(0,1));
}

void class_gen(Node node, std::ostream& line){
    // Get clas attribute and split using spaces
    std::string class_ = node.attr("class");
    if(class_.length()){
        std::vector<std::string> classes;
        boost::split(classes,class_,boost::is_any_of(" "));
        for(std::string class_ : classes){
            if(class_.length()) attr_gen(line,"."+class_);
        }
    }
}

// [foo="bar"]

sregex string = ('\"' >> *(~(set='\r','\n','\"')) >> '\"') | 
                ('\'' >> *(~(set='\r','\n','\'')) >> '\'');
sregex attr_assign = '[' >> *space >> identifier >> '=' >> string >> *space >> ']';

void attr_assign_parse(Node node, const smatch& tree){
    // Get name and value
    auto branch = tree.nested_results().begin();
    auto name = branch;
    auto value = ++branch;
    // Remove leading and trailing quotes from value
    std::string content = value->str();
    content.erase(0,1);
    content.erase(content.length()-1,1);
    // Set the attribute
    node.attr(name->str(),content);
}

void attr_assign_gen(Node node, std::ostream& line, const std::string& attr){
    attr_gen(line,"[" + attr + "=\"" + node.attr(attr) + "\"]");
}

/**
 * const: Intra-stencil dependency const declaration
 *
 * e.g. const
 */
sregex const_ = as_xpr("const");

void const_parse(Node node, const smatch& tree){
    node.attr("data-const","true");
}

void const_gen(Node node, std::ostream& line){
    auto const_ = node.attr("data-const");
    if(const_=="true") attr_gen(line,"const");
}

/**
 * hash: Intra-stencil dependency tracking hash
 *
 * e.g. &dTgy2J
 */
mark_tag hash_value(1);
sregex hash = as_xpr("&") >> (hash_value=+_w);

void hash_parse(Node node, const smatch& tree){
    node.attr("data-hash",tree[hash_value].str());
}

void hash_gen(Node node, std::ostream& line){
    auto hash = node.attr("data-hash");
    if(hash.length()) attr_gen(line,"&"+hash);
}

/**
 * off: Indicator for conditional stencil directives
 * (if,elif,else,case,default)
 */

sregex off = as_xpr("off");

void off_parse(Node node, const smatch& tree){
    node.attr("data-off","true");
}

void off_gen(Node node, std::ostream& line){
    auto off = node.attr("data-off");
    if(off.length()) attr_gen(line,"off");
}

/**
 * index: Indicator for index of each elements in 
 * a for directive
 */
mark_tag index_value(1);
sregex index = as_xpr('@') >> (index_value=+_d);

void index_parse(Node node, const smatch& tree){
    node.attr("data-index",tree[index_value].str());
}

void index_gen(Node node, std::ostream& line){
    auto index = node.attr("data-index");
    if(index.length()) attr_gen(line,"@"+index);
}

/**
 * lock: Indicator for elements that have been lock an
 * which should not be overwritten by rendering
 */

sregex lock = as_xpr("lock");

void lock_parse(Node node, const smatch& tree){
    node.attr("data-lock","true");
}

void lock_gen(Node node, std::ostream& line){
    auto lock = node.attr("data-lock");
    if(lock.length()) attr_gen(line,"lock");
}

/**
 * included: Indicator for elements that have been included
 * by an `include` directive
 */

sregex included = as_xpr("included");

void included_parse(Node node, const smatch& tree){
    node.attr("data-included","true");
}

void included_gen(Node node, std::ostream& line){
    auto included = node.attr("data-included");
    if(included.length()) attr_gen(line,"included");
}

/**
 * out: Indicator for elements that have been output
 * by a `code` directive (don't use `output` as flag since
 * that is a HTML tag name)
 */

sregex output = as_xpr("out");

void output_parse(Node node, const smatch& tree){
    node.attr("data-out","true");
}

void output_gen(Node node, std::ostream& line){
    auto output = node.attr("data-out");
    if(output.length()) attr_gen(line,"out");
}

/**
 * Stencil directives
 */

// Regexes for the types of directive arguments
// Currently, very permissive
sregex expr = +~space;
sregex selector = +(_w|"#"|"."|'-');

/**
 * Directives with no arguments
 */
sregex directive_noarg = sregex::compile("else|default");

void directive_noarg_parse(Node node, const smatch& tree){
    // Set empty directive attribute
    node.attr("data-"+tree.str(),"");
}

void directive_noarg_gen(const std::string type, Node node, std::ostream& stream){
    stream<<type;
}

/**
 * Directives with a single expression argument
 */
sregex directive_expr_name = sregex::compile("text|with|if|elif|switch|case");
sregex directive_expr = directive_expr_name >> +space >> expr;

void directive_expr_parse(Node node, const smatch& tree){
    // Get name and value
    auto branch = tree.nested_results().begin();
    auto name = branch;
    auto value = ++branch;
    // Set directive attribute
    node.attr("data-"+name->str(),value->str());
}

void directive_expr_gen(const std::string type, Node node, std::ostream& stream){
    stream<<type<<" "<<node.attr("data-"+type);
}

/**
 * Directives with a single selector argument
 */
mark_tag ref_selector(1);
sregex ref = as_xpr("ref") >> +space >> (ref_selector=selector);

void ref_parse(Node node, const smatch& tree){
    node.attr("data-ref",tree[ref_selector].str());
}

void ref_gen(Node node, std::ostream& stream){
    stream<<"ref "<<node.attr("data-ref");
}

/**
 * For directive
 */

sregex for_ = as_xpr("for") >> +space >> expr >> +space >> "in" >> +space >> expr;

void for_parse(Node node, const smatch& tree){
    // Get item and items
    auto branch = tree.nested_results().begin();
    auto item = branch;
    auto items = ++branch;
    // Set for attribute
    node.attr("data-for",item->str()+" in "+items->str());
}

void for_gen(Node node, std::ostream& stream){
    std::string attribute = node.attr("data-for");
    static const boost::regex pattern("^(\\w+) in (.+)$");
    boost::smatch match;
    if(boost::regex_search(attribute, match, pattern)) {
        std::string item = match[1].str();
        std::string items = match[2].str();
        stream<<"for "<<item<<" in "<<items;
    } else {
        STENCILA_THROW(Exception,"Syntax error in data-for attribute <"+attribute+">")
    }
}

/**
 * Include directive
 */
sregex include = as_xpr("include") >> +space >> expr >> *(+space >> selector);

void include_parse(Node node, const smatch& tree){
    auto include = tree.nested_results().begin();
    node.attr("data-include",include->str());
    auto select = ++include;
    if(select!=tree.nested_results().end()) node.attr("data-select",select->str());
}

void include_gen(Node node, std::ostream& stream){
    stream<<"include "<<node.attr("data-include");
    auto select = node.attr("data-select");
    if(select.length()) stream<<" "<<select;
}

/**
 * Set directive
 */

sregex set_ = as_xpr("set") >> +space >> identifier >> +space >> "=" >> +space >> expr;

void set_parse(Node node, const smatch& tree){
    // Get name and expression
    auto branch = tree.nested_results().begin();
    auto name = branch;
    auto expr = ++branch;
    // Set for attribute
    node.attr("data-set",name->str()+"="+expr->str());
}

void set_gen(Node node, std::ostream& stream){
    auto parts = node.attr("data-set");
    auto colon = parts.find_first_of("=");
    std::string name;
    std::string expr;
    if(colon!=std::string::npos){
        name = parts.substr(0,colon);
        expr = parts.substr(colon+1);
    } else {
        STENCILA_THROW(Exception,"Missing semicolon")
    }
    stream<<"set "<<name<<" = "<<expr;
}

/**
 * Modifier directives
 */

sregex modifier_name = sregex::compile("delete|replace|change|before|after|prepend|append");
sregex modifier = modifier_name >> +space >> selector;

void modifier_parse(Node node, const smatch& tree){
    auto branch = tree.nested_results().begin();
    auto which = branch->str();
    auto selector = (++branch)->str();
    node.attr("data-"+which,selector);
}

void modifier_gen(const std::string& which, Node node, std::ostream& stream){
    auto selector = node.attr("data-"+which);
    stream<<which<<" "<<selector;
}

/**
 * macro directive
 */

sregex macro = as_xpr("macro") >> +space >> identifier;

void macro_parse(Node node, const smatch& tree){
    auto name = tree.nested_results().begin()->str();
    node.attr("data-macro",name);
    node.attr("id",name);
}

void macro_gen(Node node, std::ostream& stream){
    stream<<"macro "<<node.attr("data-macro");
}

/**
 * par directive
 */

sregex type = +_w;
sregex par = as_xpr("par") >> +space >> identifier >> !(*space>>":">>*space>>type) >> !(*space>>"=">>*space>>expr);

void par_parse(Node node, const smatch& tree){
    std::string attribute;
    for(auto& branch : tree.nested_results()){
        auto id = branch.regex_id();
        if(id==type.regex_id()) attribute += ":";
        else if(id==expr.regex_id()) attribute += "="; 
        attribute += branch.str();
    }
    node.attr("data-par",attribute);
}

void par_gen(Node node, std::ostream& stream){
    sregex attribute = identifier >> !(":">>type) >> !("=">>expr);
    smatch tree;
    regex_match(node.attr("data-par"),tree,attribute);
    stream<<"par ";
    for(auto& branch : tree.nested_results()){
        auto id = branch.regex_id();
        if(id==type.regex_id()) stream<<":";
        else if(id==expr.regex_id()) stream<<" = ";
        stream<<branch.str();
    }
}


/**
 * Element line
 */

sregex element =
    // These grammar rules are repetitive. But attempting to simplify tem can create a rule that
    // allows nothing before the trailing text which thus implies an extra <div> which is not what is wanted
    (
        (tag >> !(+space >> (directive_noarg|directive_expr|ref|for_|include|set_|modifier|macro|par)) >> *(+space >> (id|class_|attr_assign|hash|off|index|lock|included|output))) |
        (                   (directive_noarg|directive_expr|ref|for_|include|set_|modifier|macro|par)  >> *(+space >> (id|class_|attr_assign|hash|off|index|lock|included|output))) |
        (                   (id|class_|attr_assign|hash|off|index|lock|included|output)            >> *(+space >> (id|class_|attr_assign|hash|off|index|lock|included|output)))

    ) 
    // Allow for trailing text.
    // Note that the first space is intentionally
    // stripped from text.
    >> !(space >> *text);

Node element_parse(Node parent, const smatch& tree, State& state){
    auto branch = tree.nested_results().begin();
    // The first branch is always a tag or an attr
    // If it is a tag use that, otherwise make it a <div>
    std::string name;
    if(branch->regex_id()==tag.regex_id()){
        name = branch->str();
    } else {
        for(auto branch : tree.nested_results()){
            const void* id = branch.regex_id();
            if(
                id==text.regex_id() or 
                id==ref.regex_id()
            ){
                name = "span";
                break;
            }
        }
        if(not name.length()) name = "div";
    }
    // Create the element
    Node node = parent.append(name);
    // Iterate over remaining branches which include attributes for the element
    // including those for stencil directives.
    // Note that since the first branch may need further processing (if it is an attribute)
    // that the branch iterator is not incremented until the end of the loop.
    while(branch!=tree.nested_results().end()){
        const void* id = branch->regex_id();
        // Directives
        if(id==directive_noarg.regex_id()) directive_noarg_parse(node,*branch);
        else if(id==directive_expr.regex_id()) directive_expr_parse(node,*branch);
        else if(id==ref.regex_id()) ref_parse(node,*branch);
        else if(id==for_.regex_id()) for_parse(node,*branch);
        else if(id==include.regex_id()) include_parse(node,*branch);
        else if(id==set_.regex_id()) set_parse(node,*branch);
        else if(id==modifier.regex_id()) modifier_parse(node,*branch);
        else if(id==macro.regex_id()) macro_parse(node,*branch);
        else if(id==par.regex_id()) par_parse(node,*branch);
        // Attributes
        else if(id==::id.regex_id()) id_parse(node,*branch);
        else if(id==class_.regex_id()) class_parse(node,*branch);
        else if(id==attr_assign.regex_id()) attr_assign_parse(node,*branch);
        // Flags
        else if(id==hash.regex_id()) hash_parse(node,*branch);
        else if(id==off.regex_id()) off_parse(node,*branch);
        else if(id==index.regex_id()) index_parse(node,*branch);
        else if(id==lock.regex_id()) lock_parse(node,*branch);
        else if(id==included.regex_id()) included_parse(node,*branch);
        else if(id==output.regex_id()) output_parse(node,*branch);
        // Text
        else if(id==text.regex_id()) text_parse(node,*branch,state);
        branch++;
    }
    return node;
}

void element_gen(Node node, std::ostream& stream,const std::string& indent){
    // Unless this is the very first content written to the stream
    // start elements on a new line with appropriate indentation
    if(stream.tellp()>0) stream<<"\n"<<indent;
    // The format of the element line can be complicated and dependent
    // upon the what came earlier on the line so use a string rather than
    // going straight to the stream
    std::ostringstream line;
    // Get element name and attributes
    std::string name = node.name();    
    auto attrs = node.attrs();
    // Check number of attributes
    if(attrs.size()==0) {
        // If this has no attributes then output the node name (this needs to be done for <div>s too 
        // otherwise you get a blank line)
        line << name;
    } else {
        // If this is not a <div> then output name
        if(name!="div") line << name;
        // Directive attributes. An element can only have one of these.
        // These need to go after the other attributes
        std::ostringstream directive;
        for(std::string attr : attrs){
            if(attr=="data-text") directive_expr_gen("text",node,directive);
            else if(attr=="data-ref") ref_gen(node,directive);
            else if(attr=="data-with") directive_expr_gen("with",node,directive);
            else if(attr=="data-if") directive_expr_gen("if",node,directive);
            else if(attr=="data-elif") directive_expr_gen("elif",node,directive);
            else if(attr=="data-else") directive_noarg_gen("else",node,directive);
            else if(attr=="data-switch") directive_expr_gen("switch",node,directive);
            else if(attr=="data-case") directive_expr_gen("case",node,directive);
            else if(attr=="data-default") directive_noarg_gen("default",node,directive);
            else if(attr=="data-for") for_gen(node,directive);
            else if(attr=="data-include") include_gen(node,directive);
            else if(attr=="data-set") set_gen(node,directive);
            #define MOD_(which) else if(attr=="data-"#which) modifier_gen(#which,node,directive);
                MOD_(delete)
                MOD_(replace)
                MOD_(change)
                MOD_(before)
                MOD_(after)
                MOD_(prepend)
                MOD_(append)
            #undef MOD_
            else if(attr=="data-macro") macro_gen(node,directive);
            else if(attr=="data-par") par_gen(node,directive);
            // If one of these directives has been hit then add to line
            // and break from attr loop
            if(directive.tellp()>0){
                if(line.tellp()>0) line << " ";
                line << directive.str();
                break;
            }
        }
        // id
        id_gen(node,line);
        // class
        class_gen(node,line);
        // Other attributes go before flags and directives
        for(std::string attr : node.attrs()){
            if(
                attr!="id" and attr!="class" and
                not Stencil::flag(attr) and
                not Stencil::directive(attr)
            ){
                attr_assign_gen(node,line,attr);
            }
        }
        // Flags last
        hash_gen(node,line);
        off_gen(node,line);
        index_gen(node,line);
        lock_gen(node,line);
        included_gen(node,line);
        output_gen(node,line);
    }
    // Add line to the stream
    stream<<line.str();
    
    // If only one child that is text and less than 80 characters
    // then put on the same line...
    if(node.children().size()==1){
        auto first = node.first();
        if(first.is_text()){
            auto text = first.text();
            if(text.length()<=80){
                stream<<" "<<text;
                return;
            }
        }
    }
    // ...otherwise, generate Cila for children indented one level
    for(Node child : node.children()) generate(child,stream,indent+"\t");
}

/**
 * Code directive for embedded code
 */
sregex lang = as_xpr("py") | "r";
sregex format = as_xpr("text") | "svg" | "png" | "jpg";
sregex size = +_d >> "x" >> +_d >> !(as_xpr("px") | "cm" | "in");
sregex code = lang >> !(+space >> format >> !(+space >> !size)) >> *(*space >> (const_|hash));

Node code_parse(Node parent, const smatch& tree, State& state){
    // The code language is always the first branch
    auto language = tree.nested_results().begin()->str();
    // Append the element. Use a <pre> element since this retains whitespace
    // formatting when parsed as HTML
    Node node = parent.append("pre",{{"data-code",language}});
    // Iterate over branches adding arguments
    for(auto branch : tree.nested_results()){
        auto id = branch.regex_id();
        if(id==format.regex_id()) node.attr("data-format",branch.str());
        else if(id==size.regex_id()) node.attr("data-size",branch.str());
        else if(id==const_.regex_id()) const_parse(node,branch);
        else if(id==hash.regex_id()) hash_parse(node,branch);
    }
    // Turn on code mode processing
    code_mode_start(language,state);
    return node;
}

void code_gen(Node node, std::ostream& stream, const std::string& indent){
    // Unless this is the very first content written to the stream
    // start on a new line with appropriate indentation
    if(stream.tellp()>0) stream<<"\n"<<indent;
    // Output language code; no element name
    stream<<node.attr("data-code");
    //Optional arguments
    for(auto attr : {"data-format","data-size"}){
        if(node.attr(attr).length()){
            stream<<" "<<node.attr(attr);
        }
    }
    // Hash
    const_gen(node,stream);
    hash_gen(node,stream);
    // Get the code from the first child nodes
    // Usually there will be only one, but in case there are more
    // add them all
    // Note that the text() method unencodes HTML special characters
    // e.g. &lt; for us
    std::string code;
    for(Node child : node.children()) code += child.text();
    // Normally code will start and end with a newline (that is how it is created when parsed)
    // so remove those for consistent Cila generation
    if(code[0]=='\n') code = code.substr(1);
    if(code[code.length()-1]=='\n') code = code.substr(0,code.length()-1);
    // Split into lines
    std::vector<std::string> lines;
    boost::split(lines,code,boost::is_any_of("\n"));
    // Start a new line, add extra indentation to each line
    stream<<"\n";
    for(unsigned int index=0;index<lines.size();index++){
        stream<<indent+"\t"<<lines[index];
        // Don't put a newline on last line - that is the 
        // repsonsibility of the following element
        if(index<(lines.size()-1)) stream<<"\n";
    }
}

/**
 * Equations
 */

mark_tag equation_content(1);
sregex asciimath = '|' >> (equation_content=*(~(set='|'))) >> '|';
sregex tex = as_xpr("\\(") >> (equation_content=*_) >> as_xpr("\\)");
sregex equation = asciimath|tex;

Node equation_parse(Node parent,const smatch& tree){
    // Resolve type of math
    std::string type;
    auto branch = tree.nested_results().begin();
    auto id = branch->regex_id();
    if(id==asciimath.regex_id()) type = "math/asciimath";
    else if(id==tex.regex_id()) type = "math/tex";
    // Get the math content
    std::string content = (*branch)[equation_content];
    // Create a MathJax script tag
    //  http://docs.mathjax.org/en/latest/model.html#mathjax-script-tags
    Node node = parent.append("p",{{"class","equation"}});
    node.append("script",{{"type",type+"; mode=display"}},content);
    return node;
}

void equation_gen(Node node, std::ostream& stream, const std::string& indent){
    // When generating Cila for an equation element...
    // ...get the script element
    Node script = node.select("script");
    if(script){
        // ...get all the text content
        std::string content = script.text();
        // ...add corresponding delimeters as required
        std::string begin, end;
        std::string type = script.attr("type");
        if(type=="math/asciimath; mode=display"){
            begin = end = '|';
        }
        else if(type=="math/tex; mode=display"){
            begin = "\\(";
            end = "\\)";
        }
        stream<<begin<<content<<end<<"\n";
    }
}

/**
 * Comments.
 * Currently not implemented
 */

sregex comment_text = *_;
sregex comment = as_xpr("//") >> comment_text;

/**
 * Root regex for each line
 */
sregex root = comment|equation|code|element|header|ul|ol|text;

void parse(Node node, std::istream& stream){
    // Keep track of state variables for 
    // sending to other nose-specific parsing functions
    State state;
    // Define a parent node, starting off as the stencil's
    // content root node
    Node parent = node;
    // Define current node that may become parent
    Node current = node;
    // Define a stack of <indent,Node> pairs
    // when the indentation increases then then `currrent`
    // becomes parent
    std::deque<std::pair<int,Node>> levels;
    levels.push_back({0,node});
    // Iterate over lines
    unsigned int count = 0;
    while(true){
        // Read line in
        std::string line;
        bool ok = std::getline(stream,line,'\n');
        // Check for end of file so that the end of file can be treated
        // as a "pseudo"-line to finish off processing
        if(not ok){
            if(stream.eof()) state.end = true;
            else break;
        } else {
            // Increment the line counter
            count++;
        }
        // Determine indentation and emptiness of line
        bool blank = true;
        int indentation = 0;
        for(char c : line){
            if(c=='\t') indentation++;
            else {
                blank = false;
                break;
            }
        }
        state.current.blank = blank;
        state.current.indentation = indentation;

        // If in `code_mode` then process the line immeadiately
        // and potentially change to `normal_mode`
        // This should be done before any changes to `parent`
        if(state.mode==code_mode) code_mode_check(parent,line,state);

        // Determine the parent-child relationships for this node based on its indentation
        // If indentation has increased, the current node becomes the parent
        int last = levels.back().first;
        if(indentation>last){
            levels.push_back({indentation,current});
            parent = current;
        }
        // if it has not changed, then do nothing
        else if(indentation==last){
        }
        // if it less, and the line is not blank, then pop off parents until we get to the right level
        else {
            if(not blank){
                while(levels.size()>1 and indentation<levels.back().first) levels.pop_back();
                parent = levels.back().second;
            }
        }

        // Normal mode processing which may have been turned
        // back on by `code_process` etc
        if(state.mode==normal_mode){
            // If the line has content...
            if(not state.current.blank){
                // Remove indentation before parsing
                std::string content = line.substr(indentation);
                // Parse the line into a syntax tree
                smatch tree;
                regex_match(content,tree,root);
                // Parse the first and only branch of the tree
                auto branch = tree.nested_results().begin();
                const void* id = branch->regex_id();
                if(id==comment.regex_id()) {}
                else if(id==equation.regex_id()) current = equation_parse(parent,*branch);
                else if(id==code.regex_id()) current = code_parse(parent,*branch,state);
                else if(id==element.regex_id()) current = element_parse(parent,*branch,state);
                // Markdown-like syntax
                else if(id==header.regex_id()) current = header_parse(parent,*branch,state);
                else if(id==ul.regex_id()){
                    if(parent.name()!="ul") parent = parent.append("ul");
                    current = ul_parse(parent,*branch,state);
                }
                else if(id==ol.regex_id()){
                    if(parent.name()!="ol") parent = parent.append("ol");
                    current = ol_parse(parent,*branch,state);
                }
                // Plain old text
                else if(id==text.regex_id()) current = text_parse(parent,*branch,state);
                else  STENCILA_THROW(Exception,"<cila> : "+boost::lexical_cast<std::string>(count)+": unrecognised syntax :"+line);
            }
        }
        // If this is the end then break out,
        // otherwise update state.previous
        if(state.end) break;
        else state.previous = state.current;
    }
}

void generate(Node node, std::ostream& stream, std::string indent="") {
    std::string name = node.name();
    // Generate Cila for a Stencil::Node
    if(node.is_document()){
        // .... generate Cila for each cild with no indentation
        for(Node child : node.children()) generate(child,stream);
    }
    else if(name=="code") mono_gen(node,stream);
    else if(name=="em") emphasis_gen(node,stream);
    else if(name=="strong") strong_gen(node,stream);
    else if(name=="a" and node.attr("href")!="" and node.attrs().size()==1){
        link_gen(node,stream);
    }
    else if(name=="span" and node.attr("class").find("math")!=std::string::npos){
        math_gen(node,stream);
    }
    else if(name=="p" and node.attr("class").find("equation")!=std::string::npos){
        equation_gen(node,stream,indent);
    }
    else if(node.attr("data-code")!="") code_gen(node,stream,indent);
    else if(node.is_element()) element_gen(node,stream,indent);
    else if(node.is_text()){
        std::string text = node.text();
        stream<<"\n"<<indent<<text;
    }
}

} // namespace <anonymous>

namespace Stencila {

// Parsing methods

Stencil& Stencil::cila(const std::string& string){
    // Convert the std::string to a std::istream
    // and pass to cila(std::istream&)
    std::stringstream stream(string);
    stream.seekg(0);
    return cila(stream);
}

Stencil& Stencil::cila(std::istream& stream){
    // Clear the stencil of all existing content
    clear();
    // Parse Cila with this Stencil as the root Node
    parse(*this,stream);
    return *this;
}

// Generating methods

std::string Stencil::cila(void) const {
    // Create a std::ostream and pass to cila(std::ostream)
    std::ostringstream stream;
    cila(stream);
    return stream.str();
}

std::ostream& Stencil::cila(std::ostream& stream) const {
    // Generate Cila with this Stencil as the root Node
    generate(*this,stream);
    return stream;
}

}
