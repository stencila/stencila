#include <string>

#include <boost/filesystem.hpp>

#include <stencila/component.hpp>
#include <stencila/stencil.hpp>
#include <stencila/theme.hpp>
#include <stencila/string.hpp>

namespace Stencila {

Component::Class Component::classes_[Component::types_];
std::map<std::string,Component::Instance> Component::instances_;

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
}

const Component::Class& Component::class_(Type type){
	const Class& clas = classes_[type];
	if(not clas.defined) STENCILA_THROW(Exception,"Class with type enum has not been defined.\n  type: "+type_name(type));
	return clas;
}

Component& Component::hold(Type type) {
	instances_[address(true)] = {type,this};
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

Component::Type Component::type(const std::string& path_string){
	boost::filesystem::path path(path_string);
	for(auto file : {"stencil.html","stencil.cila"}){
		if(boost::filesystem::exists(path/file)) return StencilType;
	}
	for(auto file : {"theme.css","theme.scss","theme.js"}){
		if(boost::filesystem::exists(path/file)) return ThemeType;
	}
	return NoneType;
}

std::string Component::type_name(const Component::Type& type){
	switch(type){
		case NoneType: return "None";
		case ComponentType: return "Component";
		case StencilType: return "Stencil";
		case ThemeType: return "Theme";
		case PythonContextType: return "PythonContext";
		case RContextType: return "RContext";
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
		} else if(type==ComponentType){
			component = new Component;
		} else if(type==StencilType){
			Stencil* stencil = new Stencil;
			stencil->read(path);
			component = stencil;
		} else if(type==ThemeType){
			Theme* theme = new Theme;
			theme->read(path);
			component = theme;
		} else {
			STENCILA_THROW(Exception,"Type of component at path is not currently handled by `Component::get`.\n  path: "+path+"\n  type: "+type_name(type));
		}
		component->path(path);
		component->hold(type);
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
