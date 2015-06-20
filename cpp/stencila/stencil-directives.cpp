#include <boost/regex.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

const std::vector<std::string> Stencil::directives = {
	"data-exec",
	"data-when",
	"data-attr",
	"data-text",
	"data-icon",
	"data-refer",
	"data-with",
	"data-if","data-elif","data-else",
	"data-switch","data-case","data-default",
	"data-for","data-each",
	"data-par",
	"data-set",
	"data-include","data-delete","data-replace","data-change","data-before","data-after","data-prepend","data-append",
	"data-macro",
	"data-comments","data-comment"
};

const std::vector<std::string> Stencil::flags = {
	"data-error","data-hash","data-off","data-lock",
	"data-index","data-output","data-included"
};

bool Stencil::directive(const std::string& attr){
	return std::find(directives.begin(),directives.end(),attr)!=directives.end();
}

bool Stencil::flag(const std::string& attr){
	return std::find(flags.begin(),flags.end(),attr)!=flags.end();
}

void Stencil::strip(Node node){
	// Remove attributes added during rendering
	for(std::string attr : {"data-error","data-hash","data-off"}){
		for(Node child : node.filter("["+attr+"]")) child.erase(attr);
	}
	// Remove elements added during rendering
	for(Node child : node.filter("[data-index],[data-output],[data-included],[data-label]")){
		child.destroy();
	}
	// Clear elements with text or children added during rendering
	for(Node child : node.filter("[data-text],[data-refer],#outline")){
		child.clear();
	}
}

Stencil& Stencil::strip(void){
	strip(*this);
	return *this;
}

void Stencil::crush(Node node){
	// Remove elements : `exec` elements (which contain code) and elements that
	// have been turned off `[data-off`]
	for(Node child : node.filter("[data-exec],[data-off]")){
		child.destroy();
	}	
	// Remove all directive and flag attributes
	auto all = directives;
	all.insert(all.end(),flags.begin(),flags.end());
	for(std::string attr : all){
		for(Node child : node.filter("["+attr+"]")) child.erase(attr);
	}
	// Note that no clearing of elements is done here so that the contents of
	// `write`, `refer` etc directives are retained
}

Stencil& Stencil::crush(void){
	crush(*this);
	return *this;
}

namespace {
	template<class Type>
	std::vector<Type> directives_list(const Stencil& stencil, const std::string& type) {
		std::vector<Type> directives;
		for(auto elem : stencil.filter("[data-"+type+"]")){
			Type directive(elem);
			directives.push_back(directive);
		}
		return directives;
	}
}

void Stencil::error(Node node, const std::string& type, const std::string& data){
	auto value = type;
	if(data.length()){
		auto data_clean = data;
		boost::replace_all(data_clean,"(","[");
		boost::replace_all(data_clean,")","]");
		boost::replace_all(data_clean,"\n","\\n");
		value += "(" + data_clean + ")";
	}
	node.attr("data-error",value);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Execute::Execute(void){
}

Stencil::Execute::Execute(const std::string& attribute){
	parse(attribute);
}

Stencil::Execute::Execute(Node node){
	parse(node);
}

void Stencil::Execute::parse(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern(
		"^" \
		"(\\w+(\\s*,\\s*\\w+)*)" \
		"(((eval)\\s+)?\\s+format\\s+(.+?))?" \
		"(((eval)\\s+)?\\s+width\\s+(.+?))?" \
		"(((eval)\\s+)?\\s+height\\s+(.+?))?" \
		"(((eval)\\s+)?\\s+units\\s+(.+?))?" \
		"(((eval)\\s+)?\\s+size\\s+(.+?))?" \
		"(\\s+(const))?" \
		"(\\s+(show))?" \
		"$"
	);
	if(boost::regex_search(attribute, match, pattern)) {
		valid = true;
		
		contexts = split(match[1].str(),",");
		for(auto& context : contexts) trim(context);
		for(const auto& context : contexts){
			if(not(
				context=="map" or
				context=="exec" or
				context=="cila" or
				context=="js" or
				context=="py" or
				context=="r"
			)) throw DirectiveException("context-invalid",context);
		}

		format.eval = match[5].str()=="eval";
		format.expr = match[6].str();
		width.eval = match[9].str()=="eval";
		width.expr = match[10].str();
		height.eval = match[13].str()=="eval";
		height.expr = match[14].str();
		units.eval = match[17].str()=="eval";
		units.expr = match[18].str();
		size.eval = match[21].str()=="eval";
		size.expr = match[22].str();
		constant = match[24].str()=="const";
		show = match[26].str()=="show";
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Execute::parse(Node node){
	parse(node.attr("data-exec"));
}

void Stencil::Execute::render(Stencil& stencil, Node node, Context* context){
	parse(node);

	// Check that the context accepts the declared contexts types
	bool accepted = false;
	if(contexts.size()==1 and contexts[0]=="exec") accepted = true;
	for(std::string& item : contexts){
		if(context->accept(item)){
			accepted = true;
			break;
		}
	}
	if(not accepted) return;
	
	// Create a key string for this node which starts with the current value
	// for the current cumulative hash and its attributes and text
	std::string key = stencil.hash_;
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
	if(not constant) stencil.hash_ = hash;
	// If there is no change in the hash then return
	// otherwise replace the hash (may be missing) and keep rendering
	std::string current = node.attr("data-hash");
	if(hash==current) return;
	else node.attr("data-hash",hash);

	// Get code and return if zero length
	std::string code = node.text();
	if(code.length()==0) return;

	// Evaluate parameters within context and check their values
	format.evaluate(context);
	if(format.value.length() and not(
		format.value=="text" or 
		format.value=="png" or format.value=="jpg" or format.value=="svg"
	)) throw DirectiveException("format-invalid",format.value);

	width.evaluate(context);
	height.evaluate(context);
	units.evaluate(context);

	size.evaluate(context);
	if(size.value.length()){
		static const boost::regex pattern("^([0-9]*\\.?[0-9]+)x([0-9]*\\.?[0-9]+)(\\w+)?$");
		boost::smatch match;
		if(boost::regex_search(size.value, match, pattern)){
			width.value = match[1].str();
			height.value = match[2].str();
			units.value = match[3].str();
		} else {
			throw DirectiveException("size-invalid",size.value);
		}
	}

	if(not width.value.length()) width.value = "17";
	if(not height.value.length()) height.value = "17";

	if(units.value.length()){
		if(not(
			units.value=="cm" or units.value=="in" or units.value=="px"
		)) throw DirectiveException("units-invalid",units.value);
	} else {
		units.value = "cm";
	}

	// Generate a unique id for this execute directive which, if possible, 
	// includes useful text as well as the unique-ifying hash
	std::string id;
	// Does the parent of this element have an id?
	id += node.parent().attr("id");
	// Does the parent of this element have a caption
	Node caption = node.parent().select("caption,figcaption");
	if(caption){
		std::string slug = slugify(caption.text(),25);
		if(id.length()) id += "-";
		id += slug;
	}
	if(id.length()){
		if(id[id.length()-1]!='-') id += "-";
	}
	id += stencil.hash_;

	// Execute code
	std::string result = context->execute(
		code,
		id,
		format.value,
		width.value,
		height.value,
		units.value
	);
	// Remove any existing output
	Node next = node.next_element();
	if(next and next.attr("data-output")=="true") next.destroy();

	// Append new output
	if(format.value.length()){
		// Append output element
		Xml::Document doc;
		Node output;
		if(format.value=="text"){
			output = doc.append("samp",result);
		}
		else if(format.value=="png" or format.value=="svg"){
			output = doc.append("img",{
				{"src",result},
				{"style","max-width:"+width.value+units.value+";max-height:"+height.value+units.value}
			});
		}
		else {
			throw DirectiveException("format-invalid",format.value);
		}
		if(output){
			// Flag output node 
			output.attr("data-output","true");
			// Create a copy immeadiately after code directive
			node.after(output);
		}
	}

	// Add a show flag if needed
	if(show) node.attr("data-show","true");
}

std::vector<Stencil::Execute> Stencil::execs(void) const {
	return directives_list<Stencil::Execute>(*this,"exec");
}

///////////////////////////////////////////////////////////////////////////////////////////////////

void Stencil::When::parse(const std::string& attribute){
	if(attribute.length()){
		contexts = split(attribute,",");
		for(auto& context : contexts) trim(context);
	}
	else throw DirectiveException("when-empty");
}

void Stencil::When::scan(Node node){
	parse(node.attr("data-when"));
}

void Stencil::When::render(Stencil& stencil, Node node, Context* context){
	scan(node);
	bool ok = false;
	for(auto& item : contexts){
		if(context->accept(item)){
			ok = true;
			break;
		}
	}
	if(ok) stencil.render_children(node,context);
	else node.attr("data-off","true");
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Attr::Attr(void){
}

Stencil::Attr::Attr(const std::string& attribute){
	parse(attribute);
}

Stencil::Attr::Attr(Node node){
	parse(node);
}

void Stencil::Attr::parse(const std::string& attribute){
	static const boost::regex pattern("^([\\w-]+)\\s+(.+)$");
	boost::smatch match;
	if(boost::regex_search(attribute, match, pattern)) {
		name = match[1].str();
		expression = match[2].str();
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Attr::parse(Node node){
	parse(node.attr("data-attr"));
}

void Stencil::Attr::render(Stencil& stencil, Node node, Context* context){
	parse(node);
	auto value = context->write(expression);
	node.attr(name,value);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Text::Text(void){
}

Stencil::Text::Text(const std::string& attribute){
	parse(attribute);
}

Stencil::Text::Text(Node node){
	parse(node);
}

void Stencil::Text::parse(const std::string& attribute){
	if(attribute.length()) expression = attribute;
	else throw DirectiveException("write-empty","");
}

void Stencil::Text::parse(Node node){
	parse(node.attr("data-text"));
}

void Stencil::Text::render(Stencil& stencil, Node node, Context* context){
	parse(node);
	if(node.attr("data-lock")!="true"){
		auto text = context->write(expression);
		node.text(text);
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::With::With(void){
}

Stencil::With::With(const std::string& attribute){
	parse(attribute);
}

Stencil::With::With(Node node){
	parse(node);
}

void Stencil::With::parse(const std::string& attribute){
	if(attribute.length()) expression = attribute;
	else throw DirectiveException("with-empty","");
}

void Stencil::With::parse(Node node){
	parse(node.attr("data-with"));
}

void Stencil::With::render(Stencil& stencil, Node node, Context* context){
	parse(node);
	context->enter(expression);
	stencil.render_children(node,context);
	context->exit();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

void Stencil::If::render(Stencil& stencil, Node node, Context* context){
	std::string expression = node.attr("data-if");
	bool hit = context->test(expression);
	if(hit){
		node.erase("data-off");
		stencil.render_children(node,context);
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
					stencil.render_children(next,context);
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
				stencil.render_children(next,context);
			}
			break;
		}
		else break;
		next = next.next_element();
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////

void Stencil::Switch::render(Stencil& stencil, Node node, Context* context){
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
					stencil.render_children(child,context);
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
				stencil.render_children(child,context);
			}
		} else {
			stencil.render(child,context);
		}
	}

	context->unmark();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::For::For(void){
}

Stencil::For::For(const std::string& attribute){
	parse(attribute);
}

void Stencil::For::parse(const std::string& attribute){
	static const boost::regex pattern("^(\\w+)\\s+in\\s+(.+)$");
	boost::smatch match;
	if(boost::regex_search(attribute, match, pattern)) {
		item = match[1].str();
		items = match[2].str();
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::For::render(Stencil& stencil, Node node, Context* context){
	parse(node.attr("data-for"));

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
		stencil.render(item,context);
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

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Parameter::Parameter(void){
}

Stencil::Parameter::Parameter(const std::string& attribute){
	parse(attribute);
}

Stencil::Parameter::Parameter(Node node){
	parse(node);
}

void Stencil::Parameter::parse(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern("^(\\w+)(\\s+type\\s+(\\w+))?(\\s+value\\s+(.+))?$");
	if(boost::regex_search(attribute, match, pattern)) {
		name = match[1].str();
		type = match[3].str();
		value = match[5].str();
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Parameter::parse(Node node){
	parse(node.attr("data-par"));
}

void Stencil::Parameter::render(Stencil& stencil, Node node, Context* context){
	parse(node);

	// Create a <label> element
	Node label = node.select("label");
	if(not label) label = node.append("label",{
		{"for",name+"-input"}
	},name);

	// Create an <input> element
	Node input = node.select("input");
	if(not input) input = node.append("input");
	// Set name
	input.attr("name",name);
	// Set id
	input.attr("id",name+"-input");
	// Set type
	if(type.length()){
		// Translate type into a valid type for HTML elements. See
		//    https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Input
		std::string input_type = type;
		if(input_type=="boolean") input_type = "checkbox";
		input.attr("type",input_type);
	}
	// Get current value, using default value if not defined
	std::string current = input.attr("value");
	if(not current.length() and value.length()){
		current = value;
		input.attr("value",current);
	}
	// Set value in the context
	if(current.length()>0){
		context->input(name,type,value);
	}
	// Render the input node
	Stencil::Input(input).render(stencil,input,context);
}

std::vector<Stencil::Parameter> Stencil::pars(void) const {
	return directives_list<Stencil::Parameter>(*this,"par");
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Set::Set(void){
}

Stencil::Set::Set(const std::string& attribute){
	parse(attribute);
}

Stencil::Set::Set(Node node){
	parse(node);
}

void Stencil::Set::parse(const std::string& attribute){
	static const boost::regex pattern("^(\\w+)\\s+to\\s+(.+)$");
	boost::smatch match;
	if(boost::regex_search(attribute, match, pattern)) {
		name = match[1].str();
		value = match[2].str();
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Set::parse(Node node){
	parse(node.attr("data-set"));
}

void Stencil::Set::render(Stencil& stencil, Node node, Context* context){
	parse(node);
	context->assign(name,value);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Include::Include(void){
}

Stencil::Include::Include(const std::string& attribute){
	parse(attribute);
}

Stencil::Include::Include(Node node){
	parse(node);
}

void Stencil::Include::parse(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern("^(((eval)\\s+)?(.+?))(\\s+select\\s+((eval)\\s+)?(.+?))?(\\s+(complete))?(\\s+(names))?$");
	if(boost::regex_search(attribute, match, pattern)) {
		address.expr = match[4].str();
		address.eval = match[3].str()=="eval";
		select.expr = match[8].str();
		select.eval = match[7].str()=="eval";
		complete = match[10].str()=="complete";
		names = match[12].str()=="names";
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Include::parse(Node node){
	parse(node.attr("data-include"));
}

void Stencil::Include::render(Stencil& stencil, Node node, Context* context){
	parse(node);

	// If this node has been rendered before then there will be 
	// a `data-included` node. If it does not yet exist then append one.
	Node included = node.select("[data-included]");
	if(not included) included = node.append("div",{{"data-included","true"}});

	// If the included node has been edited then it may have a data-lock
	// element. If it does not have one, then clear and reinclude it
	Node lock = included.select("[data-lock=\"true\"]");
	if(not lock) {
		// Clear the included node
		included.clear();
		//Obtain the included stencil...
		Node includee;
		//Check to see if this is a "self" include, otherwise obtain the includee
		address.evaluate(context);
		if(address.value==".") includee = node.root();
		else includee = Component::get(address.value).as<Stencil>();
		// ...select from it
		select.evaluate(context);
		if(select.value.length()){
			// ...append the selected nodes.
			for(Node node : includee.filter(select.value)){
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
	// `par` directives to avoid the included elements polluting the
	// main context or overwriting variables inadvertantly
	if(not names) context->enter();

	// Apply `set` directives
	// Apply all the `set`s specified in the include first. This
	// may include setting variables not specified as parameters
	// by the author of the included stencil.
	std::vector<std::string> assigned;
	for(Node set_node : node.filter("[data-set]")){
		Stencil::Set set;
		set.render(stencil,set_node,context);
		assigned.push_back(set.name);
	}
	// Now apply the included element's parameters
	bool ok = true;
	for(Node par : included.filter("[data-par]")){
		Stencil::Parameter parameter(par);
		// Check to see if it has already be assigned
		if(std::count(assigned.begin(),assigned.end(),parameter.name)==0){
			if(parameter.value.length()){
				// Assign the default_ in the new frame
				context->assign(parameter.name,parameter.value);
			} else {
				// Set an error
				error(node,"required",parameter.name);
				ok  = false;
			}
		}
		// Remove the parameter, there is no need to have it in the included node
		par.destroy();
	}

	// Render the `data-included` element
	if(ok) stencil.render_children(included,context);

	// Crush the children of the `data-included` element (not it though)
	if(not complete){
		for(auto child : included.children()) crush(child);
	}
	
	// Exit the included node
	if(not names) context->exit();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Macro::Macro(void){
}

Stencil::Macro::Macro(const std::string& attribute){
	parse(attribute);
}

Stencil::Macro::Macro(Node node){
	parse(node);
}

void Stencil::Macro::parse(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern("^[\\w-]+$");
	if(boost::regex_search(attribute, match, pattern)) {
		name = match.str();
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Macro::parse(Node node){
	parse(node.attr("data-macro"));
}

void Stencil::Macro::render(Stencil& stencil, Node node, Context* context){
	parse(node);
	// Add id to element so it can be selected
	node.attr("id",name);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

Stencil::Create::Create(void){
}

Stencil::Create::Create(const std::string& attribute){
	parse(attribute);
}

Stencil::Create::Create(Node node){
	parse(node);
}

void Stencil::Create::parse(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern("^(\\w+)\\s+from\\s+(((eval)\\s+)?(.+?))(\\s+select\\s+((eval)\\s+)?(.+?))?(\\s+(complete))?(\\s+(names))?$");
	if(boost::regex_search(attribute, match, pattern)) {
		name = match[1].str();
		address.expr = match[5].str();
		address.eval = match[4].str()=="eval";
		select.expr = match[9].str();
		select.eval = match[8].str()=="eval";
	} else {
		throw DirectiveException("syntax",attribute);
	}
}

void Stencil::Create::parse(Node node){
	parse(node.attr("data-create"));
}

void Stencil::Create::render(Stencil& stencil, Node node, Context* context){
	parse(node);

	// Enter a new named namespace.
	context->enter();

	// Apply `Set`s and `Par`s
	
	// Render the `init` div

	// Register binding
	//Node from;
	//stencil.bind(name,from);
	
	// Exit the named namespace
	context->exit();
}

}
