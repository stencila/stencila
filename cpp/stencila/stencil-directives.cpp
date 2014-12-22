#include <boost/regex.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

const std::vector<std::string> Stencil::directives = {
	"data-exec","data-format","data-size",
	"data-write",
	"data-refer",
	"data-with",
	"data-if","data-elif","data-else",
	"data-switch","data-case","data-default",
	"data-for",
	"data-include","data-version","data-select","data-set",
		"data-delete","data-replace","data-change","data-before","data-after","data-prepend","data-append",
	"data-macro","data-par",
};

const std::vector<std::string> Stencil::flags = {
	"data-const","data-hash","data-out",
	"data-off","data-index","data-lock","data-included",
	"data-error"
};

bool Stencil::directive(const std::string& attr){
	return std::find(directives.begin(),directives.end(),attr)!=directives.end();
}

bool Stencil::flag(const std::string& attr){
	return std::find(flags.begin(),flags.end(),attr)!=flags.end();
}

void Stencil::error(Node node, const std::string& type, const std::string& data){
	node.attr("data-error-" + type,data);
}

Stencil::Execute::Execute(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern(
		"^" \
		"(\\w+(\\s*,\\s*\\w+)*)" \
		"(\\s+format\\s+(.+?))?" \
		"(\\s+width\\s+(.+?))?" \
		"(\\s+height\\s+(.+?))?" \
		"(\\s+units\\s+(.+?))?" \
		"(\\s+size\\s+(.+?))?" \
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

	} else {
		throw DirectiveException("syntax",attribute);
	}
}

Stencil::Execute::Execute(Node node):
	Execute(node.attr("data-exec")){
}


void Stencil::Execute::render(Node node, Context* context, const std::string& id){
    // Check that the context accepts the declared contexts types
    bool accepted = false;
    for(std::string& item : contexts){
        if(context->accept(item)){
            accepted = true;
            break;
        }
    }
    if(not accepted) return;

    // Get code and execute it
    std::string code = node.text();
    if(code.length()>0){
        // Execute
        std::string result = context->execute(code,id,format,width,height,units);
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
}


Stencil::Parameter::Parameter(const std::string& attribute){
	boost::smatch match;
	static const boost::regex pattern("^(\\w+)(\\s+type\\s+(\\w+))?(\\s+value\\s+(.+))?$");
	if(boost::regex_search(attribute, match, pattern)) {
		valid = true;
		name = match[1].str();
		type = match[3].str();
		value = match[5].str();
	} else {
		valid = false;
	}
}

Stencil::Parameter::Parameter(Node node):
	Parameter(node.attr("data-par")){
}

void Stencil::Parameter::render(Node node, Context* context){
	Parameter par(node);
	if(valid){
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
		Stencil::Input(input).render(input,context);
	}
	else {
		Stencil::error(node,"syntax","");
	}
}

std::vector<Stencil::Parameter> Stencil::pars(void) const {
	std::vector<Stencil::Parameter> directives;
	for(auto elem : filter("[data-par]")){
		Stencil::Parameter directive(elem);
		directives.push_back(directive);
	}
	return directives;
}

}
