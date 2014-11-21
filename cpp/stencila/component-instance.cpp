#include <string>

#include <boost/filesystem.hpp>

#include <stencila/component.hpp>
#include <stencila/stencil.hpp>
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
		Stencil::call
	));
}

const Component::Class& Component::class_(Type type){
	const Class& clas = classes_[type];
	if(not clas.defined) STENCILA_THROW(Exception,"Class with type <"+string(type)+"> has not been defined");
	return clas;
}

Component& Component::hold(Type type) {
	instances_[address(true)] = {type,this};
	return *this;
}

Component::Type Component::type(const std::string& path_string){
	boost::filesystem::path path(path_string);
	for(auto file : {"stencil.html","stencil.cila"}){
		if(boost::filesystem::exists(path/file)) return StencilType;
	}
	return ComponentType;
}

Component::Instance Component::get(const std::string& address,const std::string& version,const std::string& comparison){
	Instance instance;

	auto iterator = instances_.find(address);
	if(iterator!=instances_.end()){
		instance = iterator->second;
	}
	else {
		// Try to find a component on the filesystem
		// and if not found clone it from Stencila hub
		std::string path = locate(address);
		if(path.length()==0) Component::clone(address);
		// Load the component into memory
		Component* component;
		Type type = Component::type(path);
		if(type==ComponentType){
			component = new Component;
		} else if(type==StencilType){
			Stencil* stencil = new Stencil;
			stencil->read(path);
			component = stencil;
		} else {
			STENCILA_THROW(Exception,"Unhandled component type <"+string(type)+">");
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
