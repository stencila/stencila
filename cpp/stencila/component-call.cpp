#include <stencila/component.hpp>

namespace Stencila {

std::string Component::call(const Component::Call& call) {
	auto what = call.what();
	if(what=="list():array"){
		Json::Document files;
		files.type<Json::Array>();
		for(auto file : list()) {
			//Json::Value& desc = files.push_object();
			//files.append(desc,"name",file.name);
			//files.append(desc,"type",file.type);
			files.push(file.name);
		}
		return files.dump();
	}

	// Respository methods
	else if(what=="commit(string)"){
		std::string string = call.arg(0);
		commit(string);
	}
	else if(what=="commits():array"){
		Json::Document log;
		log.type<Json::Array>();
		for(auto commit : commits()) {
			log.push(commit.name+" "+commit.email+" "+commit.message);
		}
		return log.dump();
	}

	else {
		STENCILA_THROW(Exception,"Method signature not registered for calling: "+what);
	}
	return "";
}        

}
