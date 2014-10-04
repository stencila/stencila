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

	template<class Reflector>
	StencilParser(Reflector& reflector, Html::Node& node):
		node_(node){
		reflector.reflect(*this);
	}

	template<typename Data,typename... Args>
	StencilParser& data(Data& data, const std::string& name, Args... args){    	
    	Html::Node node = node_.select("#"+name);
		if(node){
			// Matching node found, dispatch according to type of data
			value_(node,data,name,IsReflector<Data>());
		}
		return *this;
	}

private:

	template<typename Data>
	static void value_(Html::Node node, Data& data, const std::string& name, const std::true_type& is_reflector){
		// Data is a reflector so recurse into the current node 
		// using another StencilParser
		StencilParser(data,node);
	}

	template<typename Data>
	static void value_(Html::Node node, Data& data, const std::string& name, const std::false_type& is_reflector){
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

	template<class Reflector>
	StencilGenerator(Reflector& reflector, Html::Node& node):
		node_(node){
		reflector.reflect(*this);
	}

	template<typename Data,typename... Args>
	StencilGenerator& data(Data& data, const std::string& name, Args... args){
		auto node = node_.append("div",{{"id",name}},"");
		// Dispatch according to type of data
		value_(node,data,IsReflector<Data>());
		return *this;
	}

private:

	template<typename Data>
	static void value_(Html::Node node, Data& data,const std::true_type& is_reflector){
		// Data is a reflector so recurse into the current node 
		// usinganother StencilGenerator
		StencilGenerator(data,node);
	}

	template<typename Data>
	static void value_(Html::Node node, Data& data,const std::false_type& is_reflector){
		// Data is not a reflector so attempt to convert to string
		// boost::lexical_cast produces 3.1400001 for 3.14 so use boost::format instead
		node.text(str(boost::format("%s")%data));
	}

	Html::Node& node_;

}; // class StencilGenerator

}
}