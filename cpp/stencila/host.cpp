#include <boost/filesystem.hpp>

namespace Stencila {
namespace Host {

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

}
}
