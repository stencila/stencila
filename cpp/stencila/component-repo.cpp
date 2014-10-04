#include <boost/regex.hpp>

#include <stencila/component.hpp>

namespace Stencila {

Component::Repository* Component::repo(bool ensure) const {
	if(meta_){
		if(not meta_->repo){
			std::string path = Component::path(true);
			Repository* repo = new Repository;
			try{
				repo->open(path);
			}
			catch(Git::NoRepoError){
				if(ensure) repo->init(path);
				else{
					delete repo;
					repo = nullptr;
				}
			}
			meta_->repo = repo;
		}
		return meta_->repo;
	} else {
		if(ensure){
			meta_ = new Meta;
			return repo(true);
		}
		else return nullptr;
	}
}

Component& Component::clone(const std::string& address) {
	if(path().length()>0) STENCILA_THROW(Exception,"Path already set for this component; can not clone");
	if(not meta_) meta_ = new Meta;
	meta_->repo = new Repository;
	std::string path = stores()[1] + "/" + address;
	meta_->repo->clone("http://stenci.la/"+address+".git",path);
	meta_->path = path;
	return *this;
}

Component& Component::fork(const std::string& address) {
	clone(address);
	meta_->repo->remote("origin","");
	return *this;
}

std::string Component::origin(void) const {
	Repository* repo = this->repo();
	if(repo) return repo->remote("origin");
	else return "";
}

Component& Component::commit(const std::string& message) {
	std::string commit_message = message;
	if(commit_message.length()==0) commit_message = "Updated";
	// Get the name and email of the user
	std::string name = "";
	std::string email = "";
	// Write the component to ensure it has a working directory with up to date
	// contents
	write();
	// Get, or create, repository for the component and do the commit
	Repository* repo = this->repo(true);
	repo->commit(commit_message,name,email);
	return *this;
}

std::vector<Component::Commit> Component::commits(void) const {
	Repository* repo = this->repo();
	if(repo) return repo->commits();
	else return std::vector<Commit>(0);
}

std::string Component::version(void) const {
	Repository* repo = this->repo();
	if(repo){
		std::string version = repo->tag();
		if(version.length()==0) version = "";
		return version;
	}
	else return "";
}

Component& Component::version(const std::string& version,const std::string& message) {
	std::string new_version;
	std::string tag_message = message;
	std::string current_version = Component::version();

	boost::regex pattern("^(\\d+)\\.(\\d+)\\.(\\d+)$");

	auto regex_uint = [](const boost::smatch& matches,unsigned int index){
		return boost::lexical_cast<unsigned int>(std::string(matches[index].first,matches[index].second));
	};

	// Extract the semantic parts of the current version
	unsigned int current_major = 0;
	unsigned int current_minor = 0;
	unsigned int current_patch = 0;
	boost::smatch matches;
	if(boost::regex_match(current_version,matches,pattern)){
		current_major = regex_uint(matches,1);
		current_minor = regex_uint(matches,2);
		current_patch = regex_uint(matches,3);
	}

	if(version=="patch"){
		// Increment the patch number
		new_version = str(boost::format("%d.%d.%d")%current_major%current_minor%(current_patch+1));
	}
	else if(version=="minor"){
		// Increment the minor version number
		new_version = str(boost::format("%d.%d.0")%current_major%(current_minor+1));
	}
	else if(version=="major"){
		// Increment the minor version number
		new_version = str(boost::format("%d.0.0")%(current_major+1));
	}
	else {
		// Check that the supplied version is greater, or equal to the current
		// version
		unsigned int new_major,new_minor,new_patch;
		boost::smatch matches;
		if(boost::regex_match(version,matches,pattern)){
			new_major = regex_uint(matches,1);
			if(new_major<current_major) throw Exception(str(
				boost::format("Major version supplied is less than current major version (%d): %d")%current_major%new_major
			));
			new_minor = regex_uint(matches,2);
			if(new_major==current_major and new_minor<current_minor) throw Exception(str(
				boost::format("Minor version supplied is less than current minor version (%d): %d")%current_minor%new_minor
			));
			new_patch = regex_uint(matches,3);
			if(new_major==current_major and new_minor==current_minor and new_patch<current_patch) throw Exception(str(
				boost::format("Path version supplied is less than current path version (%d): %d")%current_patch%new_patch
			));
		} else {
			STENCILA_THROW(Exception,"Version supplied is not in correct format (e.g. 1.3.2): "+version);
		}
		new_version = version;
	}

	if(tag_message.length()==0) tag_message = "Versioned changed to " + new_version;
	std::string name = "";
	std::string email = "";
	// Get, or create, repository for the component and tag it.
	Repository* repo = this->repo(true);
	if(repo->head()=="<none>") STENCILA_THROW(Exception,"Component has not been commited. Please do a commit() before a version().");
	repo->tag(new_version,tag_message,name,email);
	return *this;
}

std::vector<std::string> Component::versions(void) const {
	Repository* repo = this->repo();
	if(repo){
		std::vector<std::string> versions = repo->tags();
		return versions;
	}
	else return std::vector<std::string>(0);
}

Component& Component::provide(const std::string& version) {
	// Check if the version already exists
	std::string version_path = path()+"/."+version;
	if(not boost::filesystem::exists(version_path)){
		// Check this is a valid version number 
		std::vector<std::string> vs = versions();
		if(std::count(vs.begin(),vs.end(),version)==0){
			STENCILA_THROW(Exception,"Component \""+address()+"\" does not have version \""+version+"\"");
		}
		// Checkout the the version into version_path
		// Clone this repo into a version_path
		Repository version_repo;
		version_repo.clone(path(),version_path);      
		// Checkout version
		version_repo.checkout(version);
		// Remove version .git directory
		boost::filesystem::remove_all(version_path+"/.git");
	}
	return *this;   
}

}
