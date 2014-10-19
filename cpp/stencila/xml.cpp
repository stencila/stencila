#include <boost/xpressive/xpressive_static.hpp>

#include <pugixml.hpp>

#include <stencila/exception.hpp>
#include <stencila/string.hpp>
#include <stencila/xml.hpp>

namespace Stencila {
namespace Xml {

Node::Node(void):
	pimpl_(new pugi::xml_node){
}

Node::Node(const pugi::xml_node& node):
	pimpl_(new pugi::xml_node(node)){
}

Node::Node(const Node& node):
	pimpl_(new pugi::xml_node(*node.pimpl_)){
}

Node::~Node(void) = default;

bool Node::exists(void) const {
	return not pimpl_->empty();
}

bool Node::is_document(void) const {
	return pimpl_->type()==pugi::node_document;
}

bool Node::is_element(void) const {
	return pimpl_->type()==pugi::node_element;
}

bool Node::is_text(void) const {
	return pimpl_->type()==pugi::node_pcdata;
}

bool Node::is_cdata(void) const {
	return pimpl_->type()==pugi::node_cdata;
}

std::string Node::name(void) const {
	return pimpl_->name();
}

bool Node::has(const std::string& name) const {
	return not attr_(name).empty();
}

std::string Node::attr(const std::string& name) const {
	auto attr = attr_(name);
	if(not attr.empty()) return attr.value();
	else return "";
}

Node& Node::attr(const std::string& name,const std::string& value){
	auto attr = attr_(name);
	if(not attr.empty()) attr.set_value(value.c_str());
	else pimpl_->append_attribute(name.c_str()) = value.c_str();
	return *this;
}

std::vector<std::string> Node::attrs(void) const {
	std::vector<std::string> attrs;
	for(pugi::xml_attribute attr = pimpl_->first_attribute(); attr; attr = attr.next_attribute()){
		attrs.push_back(attr.name());
	}
	return attrs;
}

Node& Node::concat(const std::string& name, const std::string& value, const std::string& separator){
	pugi::xml_attribute attr = attr_(name);
	if(not attr.empty()){
		std::string current = attr.as_string();
		std::string future;
		if(current.length()>0) future = current + separator + value;
		else future = value;
		attr.set_value(future.c_str());
	}else {
		pimpl_->append_attribute(name.c_str()) = value.c_str();
	}
	return *this;
}

Node& Node::erase(const std::string& name){
	auto attr = attr_(name);
	if(attr) pimpl_->remove_attribute(attr);
	return *this;
}

std::string Node::text(void) const {
	return pimpl_->text().get();
}

Node& Node::text(const std::string& text) {
	pimpl_->text().set(text.c_str());
	return *this;
}

Node Node::append(const Node& node) {
	return pimpl_->append_copy(*node.pimpl_);
}

Node Node::append(const Document& doc) {
	// To append a document it is necessary to append each of
	// it children (instead of just the document root) like this...
	for(auto child : doc.children()) pimpl_->append_copy(*child.pimpl_);
	return *this;
}

Node Node::append(const std::string& tag) {
	return pimpl_->append_child(tag.c_str());
}

Node Node::append(const std::string& tag, const std::string& text) {
	Node child = append(tag);
	child.pimpl_->append_child(pugi::node_pcdata).set_value(text.c_str());
	return child;
}

Node Node::append(const std::string& tag, const Attributes& attributes, const std::string& text) {
	Node child = append(tag);
	for(auto attribute : attributes){
		child.pimpl_->append_attribute(attribute.first.c_str()) = attribute.second.c_str();
	}
	if(text.length()>0) child.pimpl_->append_child(pugi::node_pcdata).set_value(text.c_str());
	return child;
}

Node Node::append_text(const std::string& text){
	Node child = pimpl_->append_child(pugi::node_pcdata);
	child.text(text);
	return child;
}

Node Node::append_cdata(const std::string& cdata){
	Node child = pimpl_->append_child(pugi::node_cdata);
	child.text(cdata);
	return child;
}

Node Node::append_comment(const std::string& comment){
	Node child = pimpl_->append_child(pugi::node_comment);
	child.pimpl_->set_value(comment.c_str());
	return child;
}

Node Node::append_xml(const std::string& xml){
	pugi::xml_document doc;
	pugi::xml_parse_result result = doc.load(xml.c_str());
	if(not result){
		STENCILA_THROW(Exception,result.description());
	}
	// To append a pugi::xml_document it is necessary to append each of
	// it children (instead of just the document root) like this...
	for(pugi::xml_node child : doc.children()) pimpl_->append_copy(child);
	return doc;
}   

Node& Node::append_children(const Node& other){
	for(Node child : other.children()) pimpl_->append_copy(*child.pimpl_);
	return *this;
}

Node Node::prepend(Node node){
	return pimpl_->prepend_copy(*node.pimpl_);
}

Node Node::prepend(const std::string& tag) {
	return pimpl_->prepend_child(tag.c_str());
}

Node Node::before(Node node){
	return pimpl_->parent().insert_copy_before(*node.pimpl_,*pimpl_);
}

Node Node::after(Node node){
	return pimpl_->parent().insert_copy_after(*node.pimpl_,*pimpl_);
}

Node& Node::remove(const Node& child){
	pimpl_->remove_child(*child.pimpl_);
	return *this;
}

Node& Node::clear(void){
	while(pimpl_->first_child()) pimpl_->remove_child(pimpl_->first_child());
	return *this;
}  

Node& Node::move(Node& to) {
	to.pimpl_->append_copy(*pimpl_);
	pimpl_->parent().remove_child(*pimpl_);
	return *this;
}  

void Node::destroy(void) {
	pimpl_->parent().remove_child(*pimpl_);
}  

Node Node::root(void){
	return pimpl_->root();
}

Nodes Node::children(void) const {
	Nodes children;
	for(auto child : pimpl_->children()) children.push_back(child);
	return children;
}

Node Node::first(void){
	return pimpl_->first_child();
}

Node Node::first_element(void){
	return pimpl_->find_child([](pugi::xml_node node){
		return node.type()==pugi::node_element;
	});
}

Node Node::last(void){
	return pimpl_->last_child();
}

Node Node::next(void){
	return pimpl_->next_sibling();
}

Node Node::next_element(void){
	pugi::xml_node sibling = pimpl_->next_sibling();
	while(sibling){
		if(sibling.type()==pugi::node_element){
			return sibling;
		}
		sibling = pimpl_->next_sibling();
	}
	return pugi::xml_node();
}

Node Node::previous(void){
	return pimpl_->previous_sibling();
}

Node Node::find(const std::string& tag) const {
	return pimpl_->find_node([&tag](Node node){return node.name()==tag;});
}

Node Node::find(const std::string& tag,const std::string& name) const {
	return pimpl_->find_node([&tag,&name](const pugi::xml_node& node){
		return node.name()==tag and not node.attribute(name.c_str()).empty();
	});
}

Node Node::find(const std::string& tag,const std::string& name,const std::string& value) const {
	return pimpl_->find_node([&tag,&name,&value](const pugi::xml_node& node){
		return node.name()==tag and node.attribute(name.c_str()).value()==value;
	});
}

Node Node::select(const std::string& selector,const std::string& type) const {
	std::string xpat;
	if(type=="css") xpat = xpath(selector);
	else if(type=="xpath") xpat = selector;
	else STENCILA_THROW(Exception,"Unknown selector type <"+type+">");
	try {
		return pimpl_->select_single_node(xpat.c_str()).node();
	} catch (const pugi::xpath_exception& e){
		STENCILA_THROW(Exception,e.what());
	}
}

Nodes Node::filter(const std::string& selector,const std::string& type) const {
	std::string xpat;
	if(type=="css") xpat = xpath(selector);
	else if(type=="xpath") xpat = selector;
	else STENCILA_THROW(Exception,"Unknown selector type <"+type+">");
	try {
		// Select nodes
		pugi::xpath_node_set selected = pimpl_->select_nodes(xpat.c_str());
		// Construct Nodes from pugi::xpath_node_set
		Nodes nodes;
		for(pugi::xpath_node_set::const_iterator it = selected.begin(); it != selected.end(); ++it){
			nodes.push_back(it->node());
		}
		return nodes;
	} catch (const pugi::xpath_exception& e){
		STENCILA_THROW(Exception,e.what());
	}
}

Node& Node::sanitize(const Whitelist& whitelist){
	// Element nodes get checked
	if(is_element()){
		// Is tag name allowed?
		bool ok = false;
		std::string tag = name();
		for(auto item : whitelist){
			if(tag==item.first){
				ok = true;
				// Are attribute names allowed?
				for(auto attr : attrs()){
					bool ok = false;
					for(auto allowed : item.second){
						if(attr==allowed){
							ok = true;
							break;
						}
					}
					// Attribute is not allowed...erase it
					if(not ok) erase(attr);
				}
				break;
			}
		}
		if(not ok) {
			// Tag name is not allowed... remove it from parent
			destroy();
		}
		else {
			// Tag name is allowed...check children
			for(Node& child : children()) child.sanitize(whitelist);
		}
	}
	// Document and text nodes are not checked, only their children (if any)
	else {
		for(Node& child : children()) child.sanitize(whitelist);
	}
	return *this;
}

std::string Node::dump(bool indent) const {
	std::ostringstream out;
	if(!indent){
		pimpl_->print(out,"",pugi::format_raw);
	} else {
		pimpl_->print(out,"\t",pugi::format_indent);
	}
	return out.str();
}

std::string Node::dump_children(bool indent) const {
	std::ostringstream out; 
	if(!indent){
		for(auto child : pimpl_->children()) child.print(out,"",pugi::format_raw);
	} else {
		for(auto child : pimpl_->children()) child.print(out,"\t",pugi::format_indent);
	}
	return out.str();
}

void Node::write(const std::string& filename,bool indent) const {
	std::ofstream out(filename);
	if(!indent){
		pimpl_->print(out,"",pugi::format_raw);
	} else {
		pimpl_->print(out,"\t",pugi::format_indent);
	}
}


// Anonymous namespace to keep things local to this compilation unit
namespace {

/*
CSS selector grammar

This is a partial implementation of the grammar described in the [W3C Recommendation](http://www.w3.org/TR/css3-selectors/#w3cselgrammar)

Some of the things that are not implemented or not fully implemented:
  * identifiers and strings (unicode, escape characters etc not dealt with)
  * pseudo-element ('::') 
  * pseudo-class (':')
  * negation ('not(..)')
  * namespaces ('foo|bar')
*/

using namespace boost::xpressive;

sregex identifier      = +(_w|'-');

sregex string          = ('\"' >> (s1=*(~as_xpr('\"'))) >> '\"')
					   | ('\'' >> (s1=*(~as_xpr('\''))) >> '\'');

sregex element         = identifier|'*';
	
sregex attr_class      = '.' >> identifier;
sregex attr_id         = '#' >> identifier;
sregex attr_exists     = ('['  >> *space >> identifier >> *space >> ']' );
sregex attr_comparison = as_xpr("=") | "~=" | "|=" | "^=" | "$=" | "*=";
sregex attr_compare    = ('['  >> *space >> identifier >> *space >> attr_comparison >> *space >> (identifier|string) >> *space >> ']');
sregex attr            = attr_class | attr_id | attr_exists | attr_compare;
	
sregex selector        = (element >> *attr)| (+attr);

sregex descendant      = +space;
sregex child           = *space >> '>' >> *space;
sregex adjacent_sibling= *space >> '+' >> *space;
sregex general_sibling = *space >> '~' >> *space;
	
sregex selectors       = selector>>!((descendant | child | adjacent_sibling | general_sibling)>>by_ref(selectors));
	
sregex group           = selectors >> *(*space >> ',' >> *space >> selectors);
	
// Parse a CSS selector into a syntax tree
smatch parse(const std::string& selector){
	smatch tree;
	bool matched = regex_search(selector,tree,group);
	//If no match, or not fully matched, then report error
	if(matched){
		std::string match = tree.str(0);
		if(match.length()!=selector.length()){
			std::string error = selector.substr(match.length());
			STENCILA_THROW(Exception,"syntax error in: "+error);
		}
	} else {
		STENCILA_THROW(Exception,"syntax error");
	}
	return tree;
}

/*
Translate the CSS syteax tree into XPath

There are several resources that describe how to convert CSS selectors to XPath selectors (
 [e.g.1](http://www.a-basketful-of-papayas.net/2010/04/css-selectors-and-xpath-expressions.html)
 [e.g.2](http://hakre.wordpress.com/2012/03/18/css-selector-to-xpath-conversion/)
 [e.g.3](http://plasmasturm.org/log/444/)
). An actively developed implementation is the [`cssselect` module for Python](http://packages.python.org/cssselect)
and that has been used here as the primary source for how to do conversion. In particular, 
the [web app of cssselect)[http://css2xpath.appspot.com/] is a useful place for checking how to do translations.
*/
std::string translate(const smatch& node,bool adjacent=false) {
	const void* id = node.regex_id();
	if(id==attr_id.regex_id()){
		std::string id = node.str(0);
		//"#id" to "id"
		id.erase(0,1);
		return "@id='" + id + "'";
	}
	else if(id==attr_class.regex_id()){
		std::string klass = node.str(0);
		//".class" to "class"
		klass.erase(0,1);
		return "@class and contains(concat(' ',normalize-space(@class),' '),' " + klass + " ')";
	}
	else if(id==attr_exists.regex_id()){
		std::string attr = node.nested_results().begin()->str(0);
		return "@"+attr+"";
	}
	else if(id==attr_compare.regex_id()){
		auto child = node.nested_results().begin();
		std::string name = child->str(0);
		std::string op = (++child)->str(0);

		auto value_node = ++child;
		std::string value;
		// If the value is a string then (i.e surrunded by quotes) then extract the
		// string contents...
		if(value_node->regex_id()==string.regex_id()) value = value_node->str(1);
		// otherwise if it is just an identifier then use that
		else value = value_node->str(0);

		if(op=="="){
			return "@" + name + "='" + value + "'";
		}
		else if(op=="~="){
			return "@" + name + " and contains(concat(' ',normalize-space(@" + name + "),' '),' " + value + " ')";
		}
		else if(op=="|="){
			return "@" + name + " and (@" + name + "='" + value + "' or starts-with(@" + name + ",'" + value + "-'))";
		}
		else if(op=="^="){
			return "@" + name + " and starts-with(@" + name + ",'" + value + "')";
		}
		else if(op=="$="){
			return "@" + name + " and substring(@" + name + ",string-length(@" + name + ")-" + 
				// XPath's substring function uses 1-based indexing so use length-1
				Stencila::string(value.length()-1) + ")='" + value + "'";
		}
		else if(op=="*="){
			return "@" + name + " and contains(@" + name + ",'" + value + "')";
		}
		return "error";
	}
	else if(id==selector.regex_id()){
		auto children = node.nested_results();
		auto attr = children.begin();
		int attrs = children.size();
		//Determine if first child node is universal (*) or not
		//If not then attrs start at second child
		const void* id = attr->regex_id();
		std::string name;
		if(id==element.regex_id()){
			name = attr->str(0);
			++attr;
			attrs -= 1;
		} else {
			name = "*";
		}
		//Iterate through attributes
		std::string attrs_xpath;
		int index = 1;
		for(;attr!=children.end();attr++){
			attrs_xpath += translate(*attr);
			if(index<attrs) attrs_xpath += " and ";
			index++;
		}
		
		std::string xpath;
		//If this is the child of an adjacent selectors node then
		//the generated Xpath needs to be different
		if(adjacent){
			xpath = "*[name()='" + name + "' and (position()=1)";
			if(attrs>0) xpath += " and " + attrs_xpath;
			xpath += ']';
		} else {
			xpath = name;
			if(attrs>0) xpath += '[' + attrs_xpath + ']';
		}
		return xpath;
	}
	else if(id==selectors.regex_id()){
		//Determine if a relation is involved
		auto children = node.nested_results();
		if(children.size()==1){
			//No relation, just a simple selector
			return translate(*children.begin(),adjacent);
		} else {
			//Determine the type of the relation (the second child of the node)
			auto branch = children.begin();
			auto left = branch;
			auto relation = ++branch;
			auto right = ++branch;
			const void* id = relation->regex_id();
			if(id==descendant.regex_id()){
				return translate(*left,adjacent)+"/descendant::"+translate(*right);
			}
			else if(id==child.regex_id()){
				return translate(*left,adjacent)+"/"+translate(*right);
			}
			else if(id==adjacent_sibling.regex_id()){
				return translate(*left,adjacent)+"/following-sibling::"+ translate(*right,true);
			}
			else if(id==general_sibling.regex_id()){
				return translate(*left,adjacent)+"/following-sibling::"+ translate(*right);
			}
			return "error";
		}
	}
	else if(id==group.regex_id()){
		//Root of sytax tree.
		std::string xpath = "descendant-or-self::";
		//Separate selectors using |
		auto children = node.nested_results();
		int n = children.size();
		int index = 1;
		for(auto i=children.begin();i!=children.end();i++){
			xpath += translate(*i);
			if(index<n) xpath += " | ";
			index++;
		}
		return xpath;
	}
	else {
		//Default is to translate each child node
		std::string xpath;
		for(auto i=node.nested_results().begin();i!=node.nested_results().end();i++){
			xpath += translate(*i);
		}
		return xpath;
	}
}

}// anonymous namespace

std::string Node::xpath(const std::string& selector) {
	return translate(parse(selector));
}

pugi::xml_attribute Node::attr_(const std::string& name) const {
	return pimpl_->find_attribute([&name](const pugi::xml_attribute& attr){
		return attr.name()==name;
	});
}

Document::Document(void){
	pimpl_.reset(new pugi::xml_document);
}

Document::Document(const std::string& html){
	pimpl_.reset(new pugi::xml_document);
	load(html);
}

Document::~Document() = default;

Node Document::doctype(const std::string& type){
	pugi::xml_node doctype = doc_()->prepend_child(pugi::node_doctype);
	doctype.set_value(type.c_str());
	return doctype;
}

Document& Document::load(const std::string& xml){
	pugi::xml_parse_result result = doc_()->load(xml.c_str());
	if(not result){
		STENCILA_THROW(Exception,result.description());
	}
	return *this;
}

Document& Document::read(const std::string& filename){
	pugi::xml_parse_result result = doc_()->load_file(filename.c_str());
	if(not result){
		STENCILA_THROW(Exception,result.description());
	}
	return *this;
}

pugi::xml_document* Document::doc_(void){
	return static_cast<pugi::xml_document*>(pimpl_.get());
}

}
}
