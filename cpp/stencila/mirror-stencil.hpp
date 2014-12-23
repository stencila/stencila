#pragma once

#include <stencila/mirror.hpp>
#include <stencila/stencil.hpp>
#include <stencila/string.hpp>
#include <stencila/traits.hpp>

namespace Stencila {
namespace Mirrors {

class StencilParser : public Mirror<StencilParser> {
public:

	StencilParser(const Html::Node& node):
		node_(node){}

	template<typename Type,typename... Args>
	StencilParser& data(Type& data, const std::string& name, Args... args){    	
		Html::Node node = node_.select("#"+name);
		if(node){
			// Matching node found, dispatch according to type of data
			data_(node,data,name,IsStructure<Type>(),IsArray<Type>());
		}
		return *this;
	}

private:

	template<typename Data>
	static void data_(Html::Node node, Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		// Data is a structure so recurse into the current node 
		// using another StencilParser
		StencilParser(node).mirror(data);
	}

	template<typename Data>
	static void data_(Html::Node node, Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		// Data is an array. Currently ignore.
	}

	template<typename Data>
	static void data_(Html::Node node, Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		// Data is not a reflector so attempt to convert node text to type
		std::string text = node.text();
		// Trim whitespace from text
		trim(text);
		try {
			data = unstring<Data>(text);
		}
		catch(...){
			STENCILA_THROW(Exception,"Error with text <"+text+"> for attribute <"+name+">");
		}
	}

	const Html::Node& node_;
}; // class StencilParser

class StencilGenerator : public Mirror<StencilGenerator> {
public:

	StencilGenerator(Html::Node& node):
		node_(node){}

	template<typename Type,typename... Args>
	StencilGenerator& data(Type& data, const std::string& name, Args... args){
		auto node = node_.append("div",{{"id",name}},"");
		// Dispatch according to type of data
		value_(node,data,IsStructure<Type>());
		return *this;
	}

private:

	template<typename Type>
	static void value_(Html::Node node, Type& data,const std::true_type& is_structure){
		// Data is a reflector so recurse into the current node 
		// usinganother StencilGenerator
		StencilGenerator(node).mirror(data);
	}

	template<typename Type>
	static void value_(Html::Node node, Type& data,const std::false_type& is_structure){
		// Data is not a structure so attempt to convert to string
		node.text(string(data));
	}

	Html::Node& node_;

}; // class StencilGenerator

}
}