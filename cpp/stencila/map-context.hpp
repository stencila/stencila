#pragma once

#include <stack>
#include <string>
#include <list>

#include <stencila/exception.hpp>
#include <stencila/context.hpp>
#include <stencila/string.hpp>

namespace Stencila {

class MapContext : public Context {
private:

	typedef std::map<std::string,std::string> Namespace;

	std::list<Namespace> namespaces_;

	void set_(const std::string& name, const std::string& value){
		namespaces_.front()[name] = value;
	}

	std::string get_(const std::string& name) const {
		for(auto& ns : namespaces_){
			auto i = ns.find(name);
			if(i!=ns.end()) return i->second;
		}
		throw Exception("Variable <"+name+"> not found");
	}

public:

	MapContext(void){
		namespaces_.push_front(Namespace());
	}

	std::string details(void) const {
		return "MapContext";
	}

	bool accept(const std::string& language) const {
		return language=="map";
	}
	
	std::string execute(const std::string& code, const std::string& id="", const std::string& format="", const std::string& width="", const std::string& height="", const std::string& units=""){
		return "";
	}
	
	std::string interact(const std::string& code, const std::string& id=""){
		return "";
	}

	void assign(const std::string& name, const std::string& expression){
		set_(name,expression);
	}

	void input(const std::string& name, const std::string& type, const std::string& value){
		set_(name,value);
	}

	std::string write(const std::string& expression){
		return get_(expression);
	}

	bool test(const std::string& expression){
		std::string value = get_(expression);
		return value.length()>0;
	}

	void mark(const std::string& expression){
		enter();
		set_("__subject__",get_(expression));
	}

	bool match(const std::string& expression){
		return get_("__subject__")==expression;
	}

	void unmark(void){
		exit();
	}

	bool begin(const std::string& item,const std::string& expression){
		enter();
		set_("__item__",item);
		std::string items_string = get_(expression);
		std::vector<std::string> items = split(items_string," ");
		set_("__items__",items_string);
		set_("__items_index__","0");
		set_("__items_size__",string(items.size()));
		return next();
	}

	bool next(void){
		int index = unstring<int>(get_("__items_index__"));
		int length = unstring<int>(get_("__items_size__"));
		if(index>=length){
			// Exit the loop namespace and return false
			exit();
			return false;
		} else {
			// Get the items and split them up
			std::string items_string = get_("__items__");
			std::vector<std::string> items = split(items_string," ");
			// Set the looping variable name
			std::string name = get_("__item__");
			set_(name,items[index]);
			// Increment the index and re-set it in the loop namespace
			index++;
			set_("__items_index__",string(index));
			return true;
		}
	}

	void enter(const std::string& expression=""){
		namespaces_.push_front(Namespace());
	}

	void exit(void){
		namespaces_.pop_front();
	}
};

}
