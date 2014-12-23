#include <boost/regex.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

const std::vector<std::string> Stencil::directives = {
	"data-exec",
	"data-write",
	"data-refer",
	"data-with",
	"data-if","data-elif","data-else",
	"data-switch","data-case","data-default",
	"data-for",
	"data-par",
	"data-set",
	"data-include","data-delete","data-replace","data-change","data-before","data-after","data-prepend","data-append",
	"data-macro",
};

const std::vector<std::string> Stencil::flags = {
	"data-const","data-hash","data-output",
	"data-show","data-off","data-index","data-lock","data-included",
	"data-error"
};

bool Stencil::directive(const std::string& attr){
	return std::find(directives.begin(),directives.end(),attr)!=directives.end();
}

bool Stencil::flag(const std::string& attr){
	return std::find(flags.begin(),flags.end(),attr)!=flags.end();
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
	node.attr("data-error-" + type,data);
}

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
		"(\\s+format\\s+(.+?))?" \
		"(\\s+width\\s+(.+?))?" \
		"(\\s+height\\s+(.+?))?" \
		"(\\s+units\\s+(.+?))?" \
		"(\\s+size\\s+(.+?))?" \
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
				context=="cila" or
				context=="py" or
				context=="r"
			)) throw DirectiveException("context-invalid",context);
		}

		format = match[4].str();
		if(format.length() and not(
			format=="text" or 
			format=="png" or format=="jpg" or format=="svg"
		)) throw DirectiveException("format-invalid",format);

		width = match[6].str();
		height = match[8].str();
		units = match[10].str();

		size = match[12].str();
		if(size.length()){
			static const boost::regex pattern("^([0-9]*\\.?[0-9]+)x([0-9]*\\.?[0-9]+)(\\w+)?$");
			boost::smatch match;
			if(boost::regex_search(size, match, pattern)){
				width = match[1].str();
				height = match[2].str();
				units = match[3].str();
			} else {
				throw DirectiveException("size-invalid",size);
			}
		}

		if(not width.length()) width = "17";
		if(not height.length()) height = "17";

		if(units.length()){
			if(not(
				units=="cm" or units=="in" or units=="px"
			)) throw DirectiveException("units-invalid",units);
		} else {
			units = "cm";
		}

		constant = match[14].str()=="const";
		show = match[16].str()=="show";

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

	// Get code and execute it
	std::string code = node.text();
	if(code.length()>0){
		// Execute
		std::string result = context->execute(code,stencil.hash_,format,width,height,units);
		// Remove any existing output
		Node next = node.next_element();
		if(next and next.attr("data-output")=="true") next.destroy();
		// Append new output
		if(format.length()){
			Xml::Document doc;
			Node output;
			if(format=="text"){
				output = doc.append("samp",result);
			}
			else if(format=="png" or format=="svg"){
				output = doc.append("img",{
					{"src",result}
				});
			}
			else {
				Stencil::error(node,"format-invalid",format);
			}
			if(output){
				// Flag output node 
				output.attr("data-output","true");
				// Create a copy immeadiately after code directive
				node.after(output);
			}
		}
	}

	// Add a show flag if needed
	if(show) node.attr("data-show","true");
}

std::vector<Stencil::Execute> Stencil::execs(void) const {
	return directives_list<Stencil::Execute>(*this,"exec");
}


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
		throw DirectiveException("syntax","");
	}
}

void Stencil::Parameter::parse(Node node){
	parse(node.attr("data-par"));
}

void Stencil::Parameter::render(Stencil& stencil, Node node, Context* context){
	parse(node);

	// Create an input element
	Node input = node.select("input");
	if(not input) input = node.append("input");
	// Set name
	input.attr("name",name);
	// Set type
	if(type.length()) input.attr("type",type);
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
		throw DirectiveException("syntax","");
	}
}

void Stencil::Set::parse(Node node){
	parse(node.attr("data-set"));
}

void Stencil::Set::render(Stencil& stencil, Node node, Context* context){
	parse(node);
	context->assign(name,value);
}

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
	static const boost::regex pattern("^(((eval)\\s+)?(.+?))(\\s+select\\s+((eval)\\s+)?(.+?))?$");
	if(boost::regex_search(attribute, match, pattern)) {
		address = match[4].str();
		address_eval = match[3].str()=="eval";
		select = match[8].str();
		select_eval = match[7].str()=="eval";
	} else {
		throw DirectiveException("syntax","");
	}
}

void Stencil::Include::parse(Node node){
	parse(node.attr("data-include"));
}

void Stencil::Include::render(Stencil& stencil, Node node, Context* context){
	parse(node);

	// Obtain string representation of include_expr
	std::string address_use;
	if(address_eval) address_use = context->write(address);
	else address_use = address;

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
		if(address_use==".") includee = node.root();
		else includee = Component::get(address_use).as<Stencil>();
		// ...select from it
		if(select.length()>0){
			std::string select_use;
			if(select_eval) select_use = context->write(select);
			else select_use = select;
			// ...append the selected nodes.
			for(Node node : includee.filter(select_use)){
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
	
	// Exit the included node
	context->exit();
}

}
