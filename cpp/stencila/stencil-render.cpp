#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <stencila/stencil.hpp>
#include <stencila/stencil-outline.hpp>
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

void Stencil::render_children(Node node, Context* context){
	for(Node child : node.children()) render(child,context);
}

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
			else if(attr=="data-exec") return Execute().render(*this,node,context);
			else if(attr=="data-write") return Write().render(*this,node,context);
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
		}
		// Render input elements
		if(tag=="input"){
			counts_["input"]++;
			return Input().render(*this,node,context);
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
	catch(const DirectiveException& exc){
		error(node,exc.type,exc.data);
	}
	catch(const std::exception& exc){
		error(node,"exception",exc.what());
	}
	catch(...){
		error(node,"exception","unknown");
	}
}

void Stencil::render_initialise(Node node, Context* context){
	hash_ = "";

	if(outline_) delete outline_;
	outline_ = new Outline;
}

void Stencil::render_finalise(Node node, Context* context){
	outline_->render();

	// Render refer directives
	for(Node ref : filter("[data-refer]")){
		ref.clear();
		std::string selector = ref.attr("data-refer");
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

} // namespace Stencila
