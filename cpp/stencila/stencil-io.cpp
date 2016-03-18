#include <algorithm>

#include <boost/filesystem.hpp>

#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

Stencil& Stencil::initialise(const std::string& from){
	std::size_t found = from.find("://");
	if(found==std::string::npos){
		// Initialised from an address or path
		if(boost::filesystem::exists(from)){
			// This is a path so read from it
			read(from);
		} else {
			// Search for address
			std::string path = Component::locate(from);
			if(path.length()) read(path);
			else STENCILA_THROW(Exception,"No stencil found with path or address <"+from+">");
		}        
	} else {
		// Initialised from some content
		std::string type = from.substr(0,found);
		std::string content = from.substr(found+3);
		if(type=="html") html(content);
		else if(type=="cila") cila(content);
		else if(type=="file") import(content);
		else STENCILA_THROW(Exception,"Unrecognised content type: " + type);
	}
	return *this;
}

namespace {
	struct restrict_walker {
		void traverse(Xml::Node& node) {
			for(auto child : node.children()){
				traverse(child);
				if(child.name()=="section") {
					for(auto grandchild : child.children()) {
						child.before(grandchild);
					}
					child.destroy();
				}
			}
		}
	};
}

Stencil& Stencil::restrict(void) {
	restrict_walker walker;
	walker.traverse(*this);
	return *this;
};

Stencil& Stencil::import(const std::string& path){
	if(not boost::filesystem::exists(path)){
		STENCILA_THROW(Exception,"File <"+path+"> not found");
	}
	std::string ext = boost::filesystem::extension(path);
	if(ext==".html" or ext==".cila"){
		std::ifstream file(path);
		std::stringstream stream;
		stream<<file.rdbuf();
		std::string content = stream.str();
		if(ext==".html") html(content); 
		else if(ext==".cila") cila(content);
	}
	else STENCILA_THROW(Exception,"File extension <"+ext+"> not valid for a Stencil");
	return *this;
}

Stencil& Stencil::export_(const std::string& path){
	std::string ext = boost::filesystem::extension(path);
	if(ext==".html" or ext==".cila"){
		std::ofstream file(path);
		if(ext==".html") file<<html(true,true); 
		else if(ext==".cila") file<<cila();
	}
	else if(ext==".docx") docx("to",path);
	else if(ext==".pdf") pdf("to",path);
	else STENCILA_THROW(Exception,"File extension <"+ext+"> not valid for a Stencil");
	return *this;
}

std::string Stencil::source(void) const {
	return source_;
}

Stencil& Stencil::source(const std::string& source){
	source_ = source;
	return *this;
}

Stencil& Stencil::read(const std::string& directory){
	namespace fs = boost::filesystem;
	// Check and set this stencil's path using `Component::read`
	Component::read(directory);
	// Search for a stencil.html and stencil.cila files
	std::vector<fs::path> files;
	for(std::string file : {"stencil.html","stencil.cila"}){
		fs::path filename = fs::path(path()) / file;
		if(fs::exists(filename)) files.push_back(filename);
	}
	// Sort by last modified time; latest at end
	std::sort(files.begin(), files.end(),[](const fs::path& p1, const fs::path& p2){
	 	return fs::last_write_time(p1) < fs::last_write_time(p2);
	});
	auto latest = files.back();
	// Set source
	source(latest.filename().string());
	// Read the newest using `import`
	if(not files.empty()) import(latest.string());
	return *this;
}

Stencil& Stencil::write(const std::string& directory){
	// Set this stencil's path using `Component::write`
	Component::write(directory);
	// Write to the source file, default to HTML
	if(source_=="stencil.cila") Component::write_to("stencil.cila",cila());
	else Component::write_to("stencil.html",html());
	return *this;
}

Stencil& Stencil::store(void) {
    write();
    Component::store();
    return *this;
}

Stencil& Stencil::restore(void) {
    Component::restore();
    read();
    return *this;
}

}
