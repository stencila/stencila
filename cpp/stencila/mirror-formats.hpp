#pragma once

#include <iostream>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/property_tree/ptree.hpp>
#include <boost/property_tree/json_parser.hpp>

#include <stencila/mirror.hpp>
#include <stencila/string.hpp>
#include <stencila/traits.hpp>

namespace Stencila {
namespace Mirrors {

/**
 * Note that `boost::property_tree::ptree` does not hold any type information and so
 * outputs all values as text. See:
 * 
 * 		https://svn.boost.org/trac/boost/ticket/9721
 * 		http://stackoverflow.com/questions/2855741/why-boost-property-tree-write-json-saves-everything-as-string-is-it-possible-to
 *
 * Another option is to use JsonCpp for all this. At present, this was implemented using
 * Boost only to reduce requirements for compiling structures.
 */
class JsonReader : public Mirror<JsonReader> {
public:

	JsonReader(std::istream& stream, bool optional=true):
		optional_(optional){
		try {
			boost::property_tree::read_json(stream,tree_);
		}
		catch(const std::exception& error){
			STENCILA_THROW(Exception,std::string("Error parsing JSON.\n  what: ")+error.what());
		}
		catch(...){
			STENCILA_THROW(Exception,std::string("Unknown error parsing JSON."));
		}
	}

	JsonReader(boost::property_tree::ptree tree, bool optional=true):
		tree_(tree),
		optional_(optional){
	}

	template<typename Type,typename... Args>
	JsonReader& data(Type& data, const std::string& name, Args... args){    	
		auto child = tree_.get_child_optional(name);
		if(child){
			data_(*child,data,name,IsStructure<Type>(),IsArray<Type>());
		} else {
			if(not optional_){
				STENCILA_THROW(Exception,"JSON does not include property.\n  name: "+name);
			}
		}
		return *this;
	}

private:

	template<typename Data>
	void data_(const boost::property_tree::ptree& tree, Data& data, const std::string& name, const std::true_type& is_structure, const std::false_type& is_array){
		// Data is a structure so recurse into the current node with another JsonReader
		JsonReader(tree).mirror(data);
	}

	template<typename Data>
	void data_(const boost::property_tree::ptree& tree, Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		// Data is an array. Currently ignore.
	}

	template<typename Data>
	void data_(const boost::property_tree::ptree& tree, Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
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
	bool optional_;

}; // class JsonReader


class JsonWriter : public Mirror<JsonWriter> {
public:

	JsonWriter(void):
		tree_(new boost::property_tree::ptree),
		root_(true){
	}

	JsonWriter(boost::property_tree::ptree* tree, const std::vector<std::string>& path, const std::string& name):
		tree_(tree),
		root_(false),
		path_(path){
			path_.push_back(name);
	}

	~JsonWriter(void){
		if(root_) delete tree_;
	}

	JsonWriter& write(std::ostream& stream){
		boost::property_tree::write_json(stream,*tree_);
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
		boost::property_tree::ptree& child = tree_->put_child(name,boost::property_tree::ptree());
		JsonWriter(&child,path_,name).mirror(data);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::true_type& is_array){
		// Data is an array. Write to a file an insert a link into JSON
		boost::filesystem::path dir = boost::join(path_,"/");
		boost::filesystem::create_directories(dir);
		boost::filesystem::path file = dir / (name + ".tsv");
		std::string filename = file.string();
		data.write(filename);
		tree_->put(name,"@file:"+filename);
	}

	template<typename Data>
	void data_(Data& data, const std::string& name, const std::false_type& is_structure, const std::false_type& is_array){
		// Data is not a reflector, so attempt to convert it to `Data` type
		tree_->put<Data>(name,data);
	}

	boost::property_tree::ptree* tree_;
	bool root_;
	std::vector<std::string> path_;

}; // class JsonWriter


}
}