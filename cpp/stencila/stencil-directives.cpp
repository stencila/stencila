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
