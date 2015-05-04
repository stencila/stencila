#pragma once

#include <iostream>

#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

#include <stencila/mirror.hpp>
#include <stencila/string.hpp>
#include <stencila/traits.hpp>

namespace Stencila {
namespace Mirrors {

class JsonReader : public Mirror<JsonReader> {
public:

	JsonReader(std::istream& stream){
		try {
			boost::property_tree::read_json(stream,tree_);
		}
		catch(const std::runtime_error& error){
			STENCILA_THROW(Exception,std::string("Error parsing JSON.\n  what: ")+error.what());
		}
	}

	JsonReader(boost::property_tree::ptree tree):
		tree_(tree){
	}

	template<typename Type,typename... Args>
	JsonReader& data(Type& data, const std::string& name, Args... args){    	
		auto child = tree_.get_child(name);
		if(child.get_value<std::string>().length()) data_(child,data,name,IsStructure<Type>(),IsArray<Type>());
		return *this;
	}

private:

	template<typename Data>
	void data_(boost::property_tree::ptree tree, Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		// Data is a structure so recurse into the current node with another JsonReader
		JsonReader(tree).mirror(data);
	}

	template<typename Data>
	void data_(boost::property_tree::ptree tree, Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		// Data is an array. Currently ignore.
	}

	template<typename Data>
	void data_(boost::property_tree::ptree tree, Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		// Data is not a reflector, so attempt to convert it to `Data` type
		try {
			data = tree.get_value<Data>();
		}
		catch(...){
			auto value = tree.get_value<std::string>();
			STENCILA_THROW(Exception,"Error converting value.\n  name: "+name+"\n  value: "+value);
		}
	}

	boost::property_tree::ptree tree_;

}; // class JsonReader


class JsonWriter : public Mirror<JsonWriter> {
public:

	JsonWriter(void){
	}

	JsonWriter(boost::property_tree::ptree tree):
		tree_(tree){
	}

	JsonWriter& write(std::ostream& stream){
		boost::property_tree::write_json(stream,tree_);
		return *this;
	}

	template<typename Type,typename... Args>
	JsonWriter& data(Type& data, const std::string& name, Args... args){    	
		data_(data,name,IsStructure<Type>(),IsArray<Type>());
		return *this;
	}

private:

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		// Data is a structure so create another node and recurse into it with another JsonWriter
		auto child = tree_.put_child(name,boost::property_tree::ptree());
		JsonWriter(child).mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		// Data is an array. Currently ignore.
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		// Data is not a reflector, so attempt to convert it to `Data` type
		tree_.put<Data>(name,data);
	}

	boost::property_tree::ptree tree_;

}; // class JsonWriter


}
}