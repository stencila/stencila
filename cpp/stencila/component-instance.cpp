#include <string>

#include <boost/filesystem.hpp>

#include <stencila/component.hpp>
#include <stencila/stencil.hpp>
#include <stencila/theme.hpp>
#include <stencila/sheet.hpp>
#include <stencila/function.hpp>

#include <stencila/string.hpp>

namespace Stencila {

Component::Class Component::classes_[Component::types_];
std::map<std::string,Component::Instance> Component::instances_;
typename Component::Instantiate Component::instantiate = nullptr;

void Component::class_(Type type, const Class& clas){
	classes_[type] = clas;
}

void Component::classes(void){
	class_(Component::StencilType, Class(
		"Stencil",
		Stencil::page,
		Stencil::request,
		Stencil::call
	));
	class_(Component::ThemeType, Class(
		"Theme",
		Theme::page,
		nullptr,
		Theme::call
	));
	class_(Component::SheetType, Class(
		"Sheet",
		Sheet::page,
		Sheet::request,
		nullptr
	));
	class_(Component::FunctionType, Class(
		"Function",
		Function::page,
		Function::request,
		nullptr
	));
}

const Component::Class& Component::class_(Type type){
	const Class& clas = classes_[type];
	if(not clas.defined) STENCILA_THROW(Exception,"Class with type enum has not been defined.\n  type: "+type_name(type));
	return clas;
}

Component& Component::hold(Type type) {
	auto this_address = address(true);
	auto iterator = instances_.find(this_address);
	if(iterator==instances_.end()){
		Instance instance = {type,this};
		instances_[this_address] = instance;
	} else {
		auto existing = iterator->second.pointer();
		if(existing!=this) {
			STENCILA_THROW(Exception, 
				"Attempting to hold another instance of a component.\n  address: " + this_address
			);
		}
	}
	return *this;
}

bool Component::held(void) const {
	auto iterator = instances_.find(address());
	return iterator!=instances_.end();
}

std::vector<std::pair<std::string,std::string>> Component::held_list(void){
	std::vector<std::pair<std::string,std::string>> list;
	for(auto instance : instances_){
		list.push_back({instance.first,type_name(instance.second.type())});
	}
	return list;
}

Component& Component::unhold(void) {
	auto iterator = instances_.find(address());
	if(iterator!=instances_.end()){
		instances_.erase(iterator);
	}
	return *this;
}

Component::Type Component::type(const std::string& path_string){
	boost::filesystem::path path(path_string);
	for(auto file : {"stencil.html","stencil.cila"}){
		if(boost::filesystem::exists(path/file)) return StencilType;
	}
	for(auto file : {"theme.css","theme.scss","theme.js"}){
		if(boost::filesystem::exists(path/file)) return ThemeType;
	}
	for(auto file : {"sheet.tsv"}){
		if(boost::filesystem::exists(path/file)) return SheetType;
	}
	for(auto file : {"function.yaml","function.yml","function.json"}){
		if(boost::filesystem::exists(path/file)) return SheetType;
	}
	return NoneType;
}

std::string Component::type_name(const Component::Type& type){
	switch(type){
		case NoneType: return "None";
		
		case ComponentType: return "Component";
		case StencilType: return "Stencil";
		case ThemeType: return "Theme";
		case SheetType: return "Sheet";
		case FunctionType: return "Function";

		case PythonContextType: return "PythonContext";

		case RContextType: return "RContext";
		case RSpreadType: return "RSpread";
		default: 
			STENCILA_THROW(Exception,"`Component::type_name` has not been configured for type.\n type  "+string(type));
		break;
	}
}

Component::Instance Component::get(const std::string& address,const std::string& version,const std::string& comparison){
	Instance instance;
	auto iterator = instances_.find(address);
	if(iterator!=instances_.end()){
		instance = iterator->second;
	}
	else {
		// Try to find a component on the filesystem...
		std::string path = locate(address);
		//...if not found clone it from Stencila hub
		if(path.length()==0) path = Component::clone(address);
		// Load the component into memory
		Component* component;
		Type type = Component::type(path);
		if(type==NoneType){
			STENCILA_THROW(Exception,"Path does not appear to be a Stencila component.\n  path: "+path);
		} else {
			if (Component::instantiate) {
				component = Component::instantiate(address, path, type_name(type));
				component->path(path);
				component->hold(type);
			} else {
				if(type==StencilType){
					component = open<Stencil>(path);
				} else if(type==ThemeType){
					component = open<Theme>(path);
				} else if(type==SheetType){
					component = open<Sheet>(path);
				} else if(type==FunctionType){
					component = open<Function>(path);
				} else {
					STENCILA_THROW(Exception,"Type of component at path is not currently handled by `Component::get`.\n  path: "+path+"\n  type: "+type_name(type));
				}
			}
		}
		instance = {type,component};
	}

	if(version.length()>0){
		if(comparison.length()==0 or comparison=="=="){
			Component& component = instance.as<Component>();
			component.provide(version);
		} else {
			STENCILA_THROW(Exception,"Version comparison operator not yet supported <"+comparison+">");
		}
	}

	return instance;
}


}
