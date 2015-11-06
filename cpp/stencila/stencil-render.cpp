#include <memory>

#include <boost/filesystem.hpp>
#include <boost/algorithm/string.hpp>
#include <boost/regex.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace {
	// Generate a unique id (used below);
	std::string id_unique(unsigned int length = 8){
		static const char chars[] = 
			"abcdefghijklmnopqrstuvwxyz"
			"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
			"0123456789";
		std::string id;
		id.resize(length);
		for(unsigned int index = 0; index < length; index++) {
	        id[index] = chars[std::rand() % (sizeof(chars) - 1)];
	    }
	    return id;
	}
}

namespace Stencila {

Stencil& Stencil::attach(std::shared_ptr<Context> context){
	context_ = context;
	return *this;
}

Stencil& Stencil::detach(void){
	context_ = nullptr;
	return *this;
}

std::string Stencil::context(void) const {
	if(context_) return context_->details();
	else return "none";
}

void Stencil::render_children(Node node, std::shared_ptr<Context> context){
	for(Node child : node.children()) render(child,context);
}

void Stencil::render(Node node, std::shared_ptr<Context> context){
	try {
		// Check for handled elements
		std::string tag = node.name();
		// For each attribute in this node...
		//...use the name of the attribute to dispatch to another rendering method
		//   Note that return is used so that only the first Stencila "data-xxx" will be 
		//   considered and that directive will determine how/if children nodes are processed
		for(std::string attr : node.attrs()){		
			if(attr=="data-exec"){
				// Exec directives check for hash changes before being rexecuted.
				// So don't remove errors or warnings, otherwise if the code has not been changed,
				// and the directive is not re-executed, these will be lost
				return Execute().render(*this,node,context);
			} else {
				// All other directives are always re-executed
				// So, remove any existing error or waring flags
				node.erase("data-error");
				node.erase("data-warning");
	
				if(attr=="data-where") return Where().render(*this,node,context);
				else if(attr=="data-attr") return Attr().render(*this,node,context);
				else if(attr=="data-text") return Text().render(*this,node,context);
				else if(attr=="data-with") return With().render(*this,node,context);

				else if(attr=="data-if") return If().render(*this,node,context);
				// Ignore `elif` and `else` elements as these are processed by `if`
				// and the `render_children` below should not necessarily be called for them
				else if(attr=="data-elif" or attr=="data-else") return;

				else if(attr=="data-switch") return Switch().render(*this,node,context);
				// Ignore `case` and `default` elements as these a processed by `switch`
				// and the `render_children` below should not necessarily be called for them
				else if(attr=="data-case" or attr=="data-default") return;

				else if(attr=="data-for") return For().render(*this,node,context);

				else if(attr=="data-include") return Include().render(*this,node,context);
				else if(attr=="data-set") return Set().render(*this,node,context);
				else if(attr=="data-par") return Parameter().render(*this,node,context);

				else if(attr=="data-macro") return Macro().render(*this,node,context);
			}
		}
		// Render input elements
		if(tag=="input"){
			counts_["input"]++;
			return Input().render(*this,node,context);
		}
		// Handle sections
		else if(tag=="section" and outline_.on){
			// Enter a sublevel
			outline_.index++;
			outline_.path.push_back(outline_.index);
			outline_.index = 0;
			outline_.list = outline_.list.append("ul");
			// Render children
			render_children(node,context);
			// Exit sublevel
			outline_.index = outline_.path.back();
			outline_.path.pop_back();
			outline_.list = outline_.list.parent();
			// Return so the render_children below is not hit
			return;
		}
		// Handle headings
		else if(tag=="h1" and outline_.on){
			// Add the <h1> title to the outline element
			// All section related 'accounting' is done in block above
			// This block just does transformation of this accounting into content
			
			// Get label for this level from the node
			std::string label = node.text();
			// Generate "path" prefix for label
			std::string path;
			for(auto parent : outline_.path){
				path += string(parent)+".";
			}
			// Generate level string for classes
			std::string level = string(outline_.path.size());
			// Check for node id, create one if needed, then add it to 
			// level for links and to the section header
			std::string id = node.attr("id");
			if(not id.length()){
				id = label;
				boost::to_lower(id);
				boost::replace_all(id," ","-");
				node.attr("id",id);
			}
			// Check for an existing label
			Node label_node = node.select(".label");
			if(not label_node){
				// Prepend a label
				label_node = node.prepend("span");
				label_node.attr("class","label");
				label_node.append("span",{{"class","path"}},path);
				label_node.append("span",{{"class","separator"}}," ");
			} else {
				// Ammend the label
				Node path_node = label_node.select(".path");
				if(not path_node) path_node = label_node.append("span",{{"class","path"}},path);
				else path_node.text(path);
			}            
			// Give class to the heading for styling
			node.attr("class","level-"+level);

			// Now append a link to outline
			Node li = outline_.list.append(
				"li",
				{{"class","level-"+level}}
			);
			li.append(
				"a",
				{{"href","#"+id}},
				path+" "+label
			);
		}
		// Handle table and figure captions
		else if(tag=="table" or tag=="figure"){
			Node caption = node.select("caption,figcaption");
			if(caption){
				// Increment the count for this caption type
				unsigned int& count = counts_[tag+"-caption"];
				count++;
				std::string count_string = string(count);
				// Set the index attribute on the node
				node.attr("data-index",count_string);
				// Add/modify label
				Node label = caption.select("[data-label]");
				if(not label){
					label = caption.prepend("span");
					label.attr("data-label",tag+"-"+count_string);
				}
				label.text(Stencila::title(tag)+" "+count_string);
			}
		}
		// If return not yet hit then process children of this element
		render_children(node,context);
	}
	catch(const DirectiveException& exc){
		error(node,exc.type,exc.data);
	}
	catch(const std::exception& exc){
		error(node,"exception",exc.what());
	}
	catch(...){
		error(node,"exception");
	}
}

Stencil& Stencil::render(std::shared_ptr<Context> context){
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
	counts_["table-caption"] = 0;
	counts_["figure-caption"] = 0;
	// Reset hash
	hash_ = "";
	// Reset outline outline
	Node outline = select("#outline");
	if(outline){
		outline.clear();
		outline_.on = true;
		if(outline.name()=="ul") outline_.list = outline;
		else outline_.list = outline.append("ul");
		outline_.index = 0;
		outline_.path.clear();
	}

	// Render root element within context
	render(*this,context);

	// Finalise rendering
	// Render refer directives
	for(Node ref : filter("[data-refer]")){
		ref.clear();
		std::string selector = ref.attr("data-refer");
		// Remove enclosing braces if necessary
		if(selector.front()=='{') selector = selector.substr(1);
		if(selector.back()=='}') selector.pop_back();
		// Attempt to find target using selector
		Node target = select(selector);
		if(target){
			auto tag = target.name();
			auto id = target.attr("id");
			auto index = target.attr("data-index");
			// Add an id if target does not yet have one (it's necessary for href below)
			if(id.length()==0){
				// Because index could change on a rerender we do not make id dependent upon it
				// (that would cause confusion and potentially conflicts)
				id = id_unique();
				target.attr("id",id);
			}
			// Create a reference string (e.g. Table 4)
			std::string reference;
			if(index.length()){
				reference = Stencila::title(tag) + " " + index;
			}
			else {
				// For now, just a dash
				reference = "-";
			}
			// Create the link
			Node a = ref.append(
				"a",
				{{"href","#"+id}},
				reference
			);
		} else {
			error(ref,"refer-missing","No matching element found");
		}
	}

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

Stencil& Stencil::refresh(void){
	return clean().render();
}

Stencil& Stencil::restart(void){
	return read().clean().render();
}

} // namespace Stencila
