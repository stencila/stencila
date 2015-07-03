#include <string>
#include <iostream>
#include <stdio.h>

#include <boost/filesystem.hpp>

#include <stencila/exception.hpp>
#include <stencila/string.hpp>

namespace Stencila {
namespace Host {

std::string env_var(const std::string& name) {
	const char* raw = std::getenv(name.c_str());
	// Necessary to check for raw==0 before attempting to construct a string from it
	return raw?raw:""; 
}

std::string user_store(void) {
	using namespace boost::filesystem;
	// Determine user's home directory. See:
	//   http://stackoverflow.com/questions/4891006/how-to-create-a-folder-in-the-home-directory
	//   http://stackoverflow.com/questions/2552416/how-can-i-find-the-users-home-dir-in-a-cross-platform-manner-using-c
	//   http://stackoverflow.com/questions/2910377/get-home-directory-in-linux-c
	// Try alternative environment variables
	auto home = env_var("HOME");
	if(not home.length()) {
		home = env_var("USERPROFILE");
	}
	if(not home.length()) {
		auto home_drive = env_var("HOMEDRIVE");
		auto home_path = env_var("HOMEPATH");
		home = home_drive+home_path;
	}
	// Fallback to current directory
	if(not home.length()) {
		home = current_path().generic_string();
	}
	// Create stencila directory within user's directory
	// Naming to fit the OS specific convention
	#if defined(__linux__)
		std::string stencila = ".stencila";
	#elif defined(_WIN32)
		std::string stencila = "Stencila";
	#endif
	auto dir = path(home) / stencila;
	if(not exists(dir)) create_directories(dir);
	return dir.generic_string();
}

std::string system_store(void) {
	using namespace boost::filesystem;
	path dir;
	#if defined(__linux__)
		dir = "/usr/lib/stencila";
	#elif defined(_WIN32)
		// Currently, no system directory defined on Windows
		dir = "";
	#endif
	return dir.generic_string();
}

std::vector<std::string> stores_;

std::vector<std::string> stores(void){
	if(stores_.size()==0){
		auto more = env_var("STENCILA_STORES");
		if(more.length()) {
			std::vector<std::string> more_stores = split(more,";");
			for(std::string store : more_stores) stores_.push_back(store);
		}
		stores_.push_back(user_store());
		// Currently, not including a system directory because appropriate
		// permissions would be needed to create it
		// stores.push_back(system_store());
	}
	return stores_;
}

std::string store_path(const std::string& address){
	if(stores_.size()==0) STENCILA_THROW(Exception,"No stores available");
	return (boost::filesystem::path(stores_[0])/address).generic_string();
}

std::string temp_dirname(void){
	auto path = boost::filesystem::temp_directory_path();
	path /= "stencila";
	boost::filesystem::create_directories(path);
	path /= boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%");
	return path.generic_string();
}

std::string temp_filename(const std::string& extension){
	auto path = boost::filesystem::temp_directory_path();
	path /= "stencila";
	boost::filesystem::create_directories(path);
	std::string pattern = "%%%%-%%%%-%%%%-%%%%";
	if(extension.length()) pattern += "." + extension;
	path /= boost::filesystem::unique_path(pattern);
	return path.generic_string();
}

}
}
