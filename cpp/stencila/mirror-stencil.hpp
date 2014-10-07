#pragma once

#include <boost/format.hpp>
#include <boost/algorithm/string.hpp>

#include <stencila/mirror.hpp>
#include <stencila/stencil.hpp>
#include <stencila/traits.hpp>

namespace Stencila {
namespace Mirrors {

class StencilParser : public Mirror<StencilParser> {
public:

	StencilParser(Html::Node& node):
		node_(node){}

	template<typename Type,typename... Args>
	StencilParser& data(Type& data, const std::string& name, Args... args){    	
    	Html::Node node = node_.select("#"+name);
		if(node){
			// Matching node found, dispatch according to type of data
			data_(node,data,name,IsStructure<Type>());
		}
		return *this;
	}

private:

	template<typename Data>
	static void data_(Html::Node node, Data& data, const std::string& name, const std::true_type& is_structure){
		// Data is a reflector so recurse into the current node 
		// using another StencilParser
		StencilParser(node).mirror(data);
	}

	template<typename Data>
	static void data_(Html::Node node, Data& data, const std::string& name, const std::false_type& is_structure){
		// Data is not a reflector so attempt to convert node text to type
		std::string text = node.text();
		// Trim whitespace from text
		boost::trim(text);
		try {
			data = boost::lexical_cast<Data>(text);
		}
		catch(const boost::bad_lexical_cast& e){
			STENCILA_THROW(Exception,str(boost::format("Error parsing text <%s> for attribute <%s>")%text%name));
		}
		catch(...){
			STENCILA_THROW(Exception,str(boost::format("Error with text <%s> for attribute <%s>")%text%name));
		}
	}

	Html::Node& node_;
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
		// Data is not a reflector so attempt to convert to string
		// boost::lexical_cast produces 3.1400001 for 3.14 so use boost::format instead
		node.text(str(boost::format("%s")%data));
	}

	Html::Node& node_;

}; // class StencilGenerator

}
}