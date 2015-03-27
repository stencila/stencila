#include <string>
#include <iostream>
#include <stdio.h>

#include <boost/filesystem.hpp>

namespace Stencila {
namespace Host {

std::string variable(const std::string& name) {
	const char* raw = std::getenv(name.c_str());
	// Necessary to check for raw==0 before attempting to construct a string from it
	return raw?raw:""; 
}

std::string current_dir(void) {
	return boost::filesystem::current_path().string();
}

std::string user_dir(void) {
	std::string home = std::getenv("HOME");
	if(not home.length()) {
		home = std::getenv("USERPROFILE");
	}
	if(not home.length()) {
		std::string home_drive = std::getenv("HOMEDRIVE");
		std::string home_path = std::getenv("HOMEPATH");
		home = home_drive+home_path;
	}
	if(not home.length()) {
		home = boost::filesystem::current_path().string();
	}
	return home + "/.stencila";
}

std::string system_dir(void) {
	std::string path = "/usr/lib/stencila";
	return path;
}

std::string temp_dirname(void){
	auto path = boost::filesystem::temp_directory_path();
	path /= ".stencila";
	boost::filesystem::create_directories(path);
	path /= boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%");
	return path.string();
}

std::string temp_filename(const std::string& extension){
	auto path = boost::filesystem::temp_directory_path();
	path /= ".stencila";
	boost::filesystem::create_directories(path);
	std::string pattern = "%%%%-%%%%-%%%%-%%%%";
	if(extension.length()) pattern += "." + extension;
	path /= boost::filesystem::unique_path(pattern);
	return path.string();
}

}
}
