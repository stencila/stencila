#include <boost/xpressive/xpressive_static.hpp>
#include <boost/xpressive/regex_compiler.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {
namespace {

/*!
 Stencila markup language : a Jade/Slim/Scaml/Haml-like language for Stencil templates

 * Jade http://jade-lang.com/
 * Slim http://slim-lang.com/
 * Haml http://haml.info
 * Scaml http://scalate.fusesource.org/documentation/scaml-reference.html
 * Dmsl https://github.com/dskinner/dmsl
 * 
 * http://www.boost.org/doc/libs/1_49_0/doc/html/xpressive/user_s_guide.html
 */ 

using namespace boost::xpressive;

//! Defines the id of each sytax component for faster matching in translate function
#define ID(name) const void* name##_ = name.regex_id();


#if 0
sregex element_name = +_w;
#elif 0
/*
List of vaild HTML5 element names from 
    http://www.w3.org/TR/html-markup/elements.html
and extracted using this python script:
    import requests
    import bs4
    page = requests.get('http://www.w3.org/TR/html-markup/elements.html').text
    elems = bs4.BeautifulSoup(page).findAll('span', {'class':'element'})
    print '|'.join('"%s"'%elem.text for elem in sorted(set(elems)))

Statically compiling element name list dramatically increases compile
times (e.g. 11s to 27s) and executable sizes (e.g. 10Mb to 80Mb).
*/
sregex element_name     = as_xpr("a")|"abbr"|"address"|"area"|"article"|"aside"|"audio"|"b"|"base"|"bdi"|"bdo"|"blockquote"|"body"|"br"|"button"|
                            "canvas"|"caption"|"cite"|"code"|"col"|"colgroup"|"command"|"datalist"|"dd"|"del"|"details"|"dfn"|"div"|"dl"|"dt"|
                            "em"|"embed"|"fieldset"|"figcaption"|"figure"|"footer"|"form"|"h1"|"h2"|"h3"|"h4"|"h5"|"h6"|"head"|"header"|"hgroup"|"hr"|"html"|
                            "i"|"iframe"|"img"|"input"|"ins"|"kbd"|"keygen"|"label"|"legend"|"li"|"link"|"map"|"mark"|"menu"|"meta"|"meter"|"nav"|"noscript"|
                            "object"|"ol"|"optgroup"|"option"|"output"|"p"|"param"|"pre"|"progress"|"q"|"rp"|"rt"|"ruby"|"s"|"samp"|"script"|"section"|
                            "select"|"small"|"source"|"span"|"strong"|"style"|"sub"|"summary"|"sup"|"table"|"tbody"|"td"|"textarea"|"tfoot"|"th"|"thead"|
                            "time"|"title"|"tr"|"track"|"u"|"ul"|"var"|"video"|"wbr";
#elif 1
/*
Dynamically compiling element name list only slightly increases compile
times (e.g. 11s to 15s) and executable sizes (e.g. 10Mb to 13Mb).
*/
sregex element_name = sregex::compile(
                            "a|abbr|address|area|article|aside|audio|b|base|bdi|bdo|blockquote|body|br|button|"
                            "canvas|caption|cite|code|col|colgroup|command|datalist|dd|del|details|dfn|div|dl|dt|"
                            "em|embed|fieldset|figcaption|figure|footer|form|h1|h2|h3|h4|h5|h6|head|header|hgroup|hr|html|"
                            "i|iframe|img|input|ins|kbd|keygen|label|legend|li|link|map|mark|menu|meta|meter|nav|noscript|"
                            "object|ol|optgroup|option|output|p|param|pre|progress|q|rp|rt|ruby|s|samp|script|section|"
                            "select|small|source|span|strong|style|sub|summary|sup|table|tbody|td|textarea|tfoot|th|thead|"
                            "time|title|tr|track|u|ul|var|video|wbr"
);
#endif
    ID(element_name)
    
sregex inlinee_expr      = *(~(set='|'));
    ID(inlinee_expr)
    
sregex inlinee           = *element_name >> "|" >> inlinee_expr >> "|";
    ID(inlinee)
    
sregex chars            = *space>>+(~(set='|',' ','\t'))>>*space;
    ID(chars)

sregex text             = +(inlinee|chars);
    ID(text)


sregex code = sregex::compile("py|r");
    ID(code)

///////////////////

sregex expr = +_;
    ID(expr)

sregex directive_for = as_xpr("for") >> +space >> expr >> +space >> "in" >> +space >> expr;
    ID(directive_for)

///////////////////

//CSS selector
sregex css_selector = +_;
    ID(css_selector)

//Stencil identifier
sregex stencil_identifier = +_w;
    ID(stencil_identifier)

///////////////////

sregex directive_include = as_xpr("include") >> +space >> stencil_identifier >> *(+space >> css_selector);
    ID(directive_include)

///////////////////

sregex directive_modifier_name = sregex::compile("replace|before|after|prepend|append");
    ID(directive_modifier_name)

sregex directive_modifier = directive_modifier_name >> +space >> css_selector;
    ID(directive_modifier)

///////////////////

sregex directive_arg_name = sregex::compile("text|with|if|elif|switch|value");
    ID(directive_arg_name)
    
sregex directive_arg = directive_arg_name >> +space >> expr;
    ID(directive_arg)

///////////////////
    
sregex directive_noarg = sregex::compile("script|else|default");
    ID(directive_noarg)

///////////////////

sregex attr_identifier       = +(_w|'-');
    ID(attr_identifier)

sregex attr_string            = ('\"' >> *(~(set='\r','\n','\"')) >> '\"') | 
                           ('\'' >> *(~(set='\r','\n','\'')) >> '\'');
    ID(attr_string)

sregex attr_class       = '.' >> attr_identifier;
    ID(attr_class)
sregex attr_id          = '#' >> attr_identifier;
    ID(attr_id)
sregex attr_assign      = attr_identifier >> '=' >> attr_string;
    ID(attr_assign)

sregex element          = (
    (*(element_name >> "!") >> (directive_include|directive_modifier|directive_for|directive_arg|directive_noarg)) |
    (element_name >> +(+space >> attr_assign)) |
    (element_name >> *(attr_class|attr_id|'[' >> *space >> +(attr_assign>>*space) >> ']')) |
                     +(attr_class|attr_id|'[' >> *space >> +(attr_assign>>*space) >> ']')
) >> *(+space >> *text);
    ID(element)

///////////////////

sregex comment_text = *_;
    ID(comment_text)
    
sregex comment = as_xpr("//") >> comment_text;
    ID(comment)

///////////////////

sregex indent = *space;
    ID(indent)

sregex line = indent >> (comment|code|element|text);
    ID(line)

#undef ID
//! @}

std::map<const void*,std::string> rules;
inline
void initialise(void) {
    static bool initialised = false;
    if(initialised) return;

    #define MAP(name) rules[name.regex_id()] = #name;
    MAP(inlinee_expr)
    MAP(inlinee)
    MAP(chars)
    MAP(text)
    
    MAP(code)
    
    
    MAP(css_selector)
    MAP(stencil_identifier)
    MAP(directive_include)
    MAP(directive_modifier)
    
    MAP(expr)
    MAP(directive_for)
    MAP(directive_arg_name)
    MAP(directive_arg)
    MAP(directive_noarg)
    
    MAP(attr_identifier)
    MAP(attr_string)
    MAP(attr_class)
    MAP(attr_id)
    MAP(attr_assign)
    
    MAP(element_name)
    MAP(element)
    
    MAP(comment)
    
    MAP(indent)
    
    MAP(line)
    #undef MAP
    initialised = true;
}


struct Line {
    std::string content;
    smatch tree;
    std::vector<Line*> children;
    
    Line(const std::string& content_=""):
        content(content_){
    }
    
    ~Line(void){
        for(auto i=children.begin();i!=children.end();i++) delete *i;
    }
    
    std::string descendent_content(void){
        std::string text;
        for(auto child=children.begin();child!=children.end();child++){
            text += (*child)->content + "\n";
            text += (*child)->descendent_content();
        }
        return text;
    }
    
    void make_top(Xml::Node node){
        for(auto child=children.begin();child!=children.end();child++){
            (*child)->make(node);
        }
    }
    
    void make(Xml::Node node){
        regex_match(content,tree,line);
        auto branch = tree.nested_results().begin();
        branch++; //Skip the indent
        const void* id = branch->regex_id();
        if(id==comment_) make_comment(node,*branch);
        else if(id==code_) make_code(node,*branch);
        else if(id==element_) make_element(node,*branch);
        else if(id==text_) make_text(node,*branch);
        else {
            for(auto child=children.begin();child!=children.end();child++){
                (*child)->make(node);
            }
        }
    }
    
    void make_comment(Xml::Node node,const smatch& tree){
        std::string comment;
        auto text = tree.nested_results().begin();
        if(text != tree.nested_results().end()) comment = text->str();
        std::string decendents = descendent_content();
        if(decendents.length()>0 and comment.length()>0) comment += "\n";
        comment += decendents;
        comment += " ";
        Xml::Document::append_comment(node,comment);
    }
    
    void make_code(Xml::Node node,const smatch& tree){
        Xml::Node self = Xml::Document::append(node,"script");
        std::string lang = tree.str();
        //Add the type attribute
        Xml::Document::set(self,"type","text/"+lang);
        //Add a comment token to escape "<![CDATA[" for HTML parsers
        if(lang=="r" or lang=="py") Xml::Document::append_text(self,"#");
        //Concatenate the code
        // A starting newline is required to escape the commented "<![CDATA[" line
        std::string code = "\n" + descendent_content();
        //Add a comment token to escape "]]>". This needs to be added to the code string!
        if(lang=="r" or lang=="py") code += "#";
        Xml::Document::append_cdata(self,code);
    }
    
    void make_element(Xml::Node node,const smatch& tree){
        auto branch = tree.nested_results().begin();
        //First branch is a element_name or an attr
        auto element_name_or_attr = branch;
        //If its an element name get it, otherwise make it div
        std::string element_name;
        if(element_name_or_attr->regex_id()==element_name_){
            element_name = element_name_or_attr->str();
        } else {
            element_name = "div";
        }
        Xml::Node self = Xml::Document::append(node,element_name);
        for(auto branch = tree.nested_results().begin();branch!=tree.nested_results().end();branch++){
            const void* id = branch->regex_id();
            auto nested = branch->nested_results().begin();
            if(id==directive_include_) {
                auto identifier = nested;
                Xml::Document::set(self,"data-include",identifier->str());
                if(branch->nested_results().size()>1){
                    auto selector = ++nested;
                    Xml::Document::set(self,"data-select",selector->str());
                }
            }
            else if(id==directive_for_) {
                auto item = nested;
                auto expr = ++nested;
                Xml::Document::set(self,"data-for",item->str()+":"+expr->str());
            }
            else if(id==directive_arg_ or id==directive_modifier_) {
                auto name = nested;
                auto arg = ++nested;
                Xml::Document::set(self,"data-"+name->str(),arg->str());
            }
            else if(id==directive_noarg_){
                Xml::Document::set(self,"data-"+branch->str(),"");
            }
            else if(id==attr_id_) {
                //Remove leading "#"
                Xml::Document::set(self,"id",branch->str(0).erase(0,1));
            }
            else if(id==attr_class_){
                //Remove leading "."
                Xml::Document::add(self,"class",branch->str(0).erase(0,1));
            }
            else if(id==attr_assign_){
                auto nested = branch->nested_results().begin();
                auto name = nested;
                auto value = ++nested;
                //Remove leading and trailing quotes
                std::string string = value->str();
                string.erase(0,1);
                string.erase(string.length()-1,1);
                Xml::Document::set(self,name->str(),string);
            }
            else if(id==text_) make_text(self,*branch);
        };
        
        for(auto child=children.begin();child!=children.end();child++){
            (*child)->make(self);
        }
    }
    
    void make_text(Xml::Node node,const smatch& tree){
        for(auto branch = tree.nested_results().begin();branch!=tree.nested_results().end();branch++){
            const void* id = branch->regex_id();
            if(id==chars_) Xml::Document::append_text(node,branch->str());
            if(id==inlinee_) make_inline(node,*branch);
        }
    }
    
    void make_inline(Xml::Node node,const smatch& tree){
        auto branch = tree.nested_results().begin();
        std::string element_name = "span";
        auto expression = branch;
        if(tree.nested_results().size()==2){
            element_name = branch->str();
            expression = ++branch;
        }
        Xml::Node self = Xml::Document::append(node,element_name);
        Xml::Document::set(self,"data-text",expression->str());
    }
    
    std::string print(std::string indent=""){
        std::string p = indent + " \"" + content + "\"\n";

        initialise();
        regex_match(content,tree,line);
        std::stringstream stream;
        print(tree,stream,indent+"  ");
        p += stream.str();
        
        for(auto i=children.begin();i!=children.end();i++){
            p += (*i)->print(indent+"  ");
        }
        return p;
    }
    
    void print(const smatch& node,std::ostream& stream,std::string indent=""){
        if(node.size()>0){
            auto regex_id = node.regex_id();
            std::string rule = rules[regex_id];
            stream<<indent<<rule<<": \""<<node.str(0)<<"\"\n";
        } else {
            stream<<indent<<"<empty>\n";
        }
        for(auto i=node.nested_results().begin();i!=node.nested_results().end();i++){
            print(*i,stream,indent+"  ");
        }
    }
};

Line parse(const std::string& stem) {
    //Create a root syntax tree node
    Line root;
    
    //Initialise structures for keeping track of parent-child relationships
    Line* parent = &root;
    Line* previous = &root;
    int current = 0;
    std::deque<std::pair<int,Line*>> levels;
    levels.push_back({-1,&root});
    
    //For each line...
    std::stringstream stream(stem);
    std::string string;
    while(std::getline(stream,string,'\n')){
        //Create a new Line
        Line* line = new Line(string);
        //Determine the parent-child relationships for this node based on its indentation
        int indent = string.find_first_not_of(" ");
        if(indent==(int)std::string::npos) indent = current;
        if(indent>levels.back().first){
            parent = previous;
            levels.push_back({indent,parent});
        } 
        else {
            while(indent<levels.back().first){
                levels.pop_back();
            }
            parent = levels.back().second;
        }
        parent->children.push_back(line);
        previous = line;
        current = indent;
    };
    
    return root;
}

} // namespace

Stencil& Stencil::from_stem(const std::string& stem){
    from_scratch();
    parse(stem).make_top(body());
    return *this;
}

std::string Stencil::stem_print(const std::string& stem){
    return parse(stem).print();
}

} //namespace Stencila
