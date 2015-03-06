#include <fstream>

#include <boost/filesystem.hpp>

#include <stencila/exception.hpp>
#include <stencila/string.hpp>
#include <stencila/helpers.hpp>

namespace Stencila {
namespace Helpers {

std::string script(const std::string& filename,const std::string& contents){
	auto dir = boost::filesystem::temp_directory_path();
	dir /= ".stencila/scripts";
	auto path = dir / filename;
	if(not boost::filesystem::exists(path)){
		boost::filesystem::create_directories(dir);
		std::ofstream file(path.string());
		file<<contents;
		file.close();
	}
	return path.string();
}

void execute(const std::string& command) {
	auto status = system(command.c_str());
	if(status != 0) STENCILA_THROW(Exception,"System call failed\n  command: "+command+"\n  status: "+string(status));
}

std::string call(const std::string& command) {
	FILE* stream  = popen(command.c_str(), "r");
	if(stream==NULL) STENCILA_THROW(Exception,"System call failed\n  command: "+command);
	std::string string;
	const int buffer_size = 1028;
	char buffer[buffer_size];
	while(fgets(buffer, buffer_size, stream) != NULL) string.append(buffer);
  	pclose(stream);
	return trim(string);
}

}
}
