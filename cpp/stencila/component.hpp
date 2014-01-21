#pragma once

#include <string>
#include <vector>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/regex.hpp>

#include <stencila/home.hpp>
#include <stencila/utilities/git.hpp>

namespace Stencila {

/**
 * Component
 */
template<class Class=void>
class Component {
public:

    typedef Utilities::Git::Repository  Repository;
    typedef Utilities::Git::Commit      Commit;
    
private:

    /**
     * Component meta data
     *
     * Encapsulated as a separate class to reduce the minimum size of a 
     * Component object to the sizeof(Meta*). 
     */
    class Meta {
    public:
        std::string title;
        std::string description;  
        std::vector<std::string> keywords;
        std::vector<std::string> authors;

        /**
         * Local filesystem path to the component.
         *
         * This is maintained, principally so that a component can `write()` itself
         * to the local filesystem without a `to` argument being supplied.
         */
        std::string path;

        /**
         * Repository for the component
         *
         * Lazily initialised
         */
        Repository* repo;

        Meta(void):
            repo(nullptr){
        }
    };

    /**
     * Metadata on the component
     *
     * Lazily initialised
     */
    mutable Meta* meta_;

public:

    Component(void):
        meta_(nullptr){
    }

    Component(const Component& other):
        meta_(nullptr){
    }

    ~Component(void){
        if(meta_) delete meta_;
    }


    Class& self(void){
        return static_cast<Class&>(*this);
    }


    /**
     * @{
     * @name Information attribute getters and setters
     */

    //Define some local get and set macros
    
    #define _GET(attr,value) \
        if(not meta_){ \
            meta_ = new Meta; \
            meta_->attr = value; \
        } \
        return meta_->attr;

    #define _SET(attr,value) \
        if(not meta_) meta_ = new Meta; \
        meta_->attr = value; \
        return self();

    /**
     * Get component title
     */
    const std::string& title(void) const {
        _GET(title,"")
    }

    /**
     * Get component title
     */
    std::string& title(void) {
        _GET(title,"")
    }

    /**
     * Set component title
     * @param title Title of the component
     */
    Class& title(const std::string& title) {
        _SET(title,title)
    }

    /**
     * Get component description
     */
    const std::string& description(void) const {
        _GET(description,"")
    }

    /**
     * Get component description
     */
    std::string& description(void) {
        _GET(description,"")
    }

    /**
     * Set component description
     * @param description Description for the component
     */
    Class& description(const std::string& description) {
        _SET(description,description)
    }
    
    /**
     * Get component keywords
     */
    const std::vector<std::string>& keywords(void) const {
        _GET(keywords,std::vector<std::string>(0))
    }

    /**
     * Get component keywords
     */
    std::vector<std::string>& keywords(void) {
        _GET(keywords,std::vector<std::string>(0))
    }

    /**
     * Set component keywords
     * @param keywords Keywords for the component
     */
    Class& keywords(const std::vector<std::string>& keywords) {
        _SET(keywords,keywords)
    }

    /**
     * Get component authors
     */
    const std::vector<std::string>& authors(void) const {
        _GET(authors,std::vector<std::string>(0))
    }

    /**
     * Get component authors
     */
    std::vector<std::string>& authors(void) {
        _GET(authors,std::vector<std::string>(0))
    }

    /**
     * Set component authors
     * @param authors Authors of the component
     */
    Class& authors(const std::vector<std::string>& authors) {
        _SET(authors,authors)
    }

    // Undefine local macros
    #undef _GET
    #undef _SET

    /**
     * @}
     */


    /**
     * @{
     * @name Persistence methods
     */
    
    /**
     * Get the Component path
     */
    const std::string& path(void) const {
        if(!meta_) meta_ = new Meta;
        return meta_->path;
    }

    /**
     * Set the Component path
     *
     * If an empty string is supplied as `path` then a unique path under the "transient"
     * subdirectory of the user's Stencila library will be created.
     * 
     * @param  path Path to component
     */
    Class& path(const std::string& path) {
        if(!meta_) meta_ = new Meta;
        std::string current_path = meta_->path;
        std::string new_path = path;
        // If the current path is empty...
        if(current_path.length()==0){
            // If the new path is empty then...
            if(new_path.length()==0){
                // generate a unique path,
                boost::filesystem::path unique_path = Stencila::library_user();
                unique_path /= "transient";
                unique_path /= boost::filesystem::unique_path("%%%%-%%%%-%%%%-%%%%");
                new_path = unique_path.string();
            }
            // and create the path.
            boost::filesystem::create_directories(new_path);
            meta_->path = new_path;
        } 
        // If the current path is not empty...
        else {
            /// and the new path is not empty...
            if(new_path.length()>0){
                // creat necessary directories for the following rename operation
                boost::filesystem::create_directories(new_path);
                // move (i.e rename) existing path to the new path.
                boost::filesystem::rename(current_path,new_path);
                meta_->path = new_path;
            }
        }
        return self();
    }
    
    /**
     * Read the Component from a directory
     *
     * Note that reading a component from path and then reading it from a different
     * path will move the component directory to the new path
     * 
     * @param from Filesystem path to component
     */
    Class& read(const std::string& from=""){
        path(from);
        self().read_();
        return self();
    }
    
    /**
     * Write the Component to a directory
     * 
     * @param to Filesystem path to component
     */
    Class& write(const std::string& to=""){
        path(to);
        self().write_();
        return self();
    }

    /**
     * Destroy the component directory
     */
    Class& destroy(void){
        if(meta_){
            std::string current_path = meta_->path;
            if(current_path.length()>0){
                if(boost::filesystem::exists(current_path)){
                    boost::filesystem::remove_all(current_path);
                }
                meta_->path = "";
            }
        }
        return self();
    }
    
    /**
     * @}
     */


    /**
     * @{
     * @name Repository interface
     */    
    
private:

    Repository* repo_(bool create = false) const {
        if(not meta_) meta_ = new Meta;
        if(not meta_->repo and create){
            meta_->repo = new Repository;
            meta_->repo->open_or_init(path());
        }
        return meta_->repo;
    }

public:
    
    Class& commit(const std::string& message="") {
        std::string commit_message = message;
        if(commit_message.length()==0) commit_message = "Updated";
        // Get the name and email of the user
        //! @todo Get the name and email of the user from ~/.stencila/.config
        std::string name = "";
        std::string email = "";
        // Write the component to ensure it has a working directory with up to date
        // contents
        write();
        // Get, or create, repository for the component and do the commit
        Repository* repo = repo_(true);
        repo->commit(commit_message,name,email);
        return self();
    }

    std::vector<Commit> log(void) const {
        Repository* repo = repo_();
        if(repo) return repo->log();
        else return std::vector<Commit>(0);
    }

    std::string version(void) const {
        Repository* repo = repo_();
        if(repo){
            std::string version = repo->tag();
            if(version.length()==0) version = "0.0.0";
            return version;
        }
        else return "0.0.0";
    }

    Class& version(const std::string& version,const std::string& message="") {
        std::string new_version;
        std::string tag_message = message;
        std::string current_version = Component<Class>::version();

        boost::regex pattern("^(\\d+)\\.(\\d+)\\.(\\d+)$");

        auto regex_uint = [](const boost::smatch& matches,uint index){
            return boost::lexical_cast<uint>(std::string(matches[index].first,matches[index].second));
        };

        // Extract the sematic parts of the current version
        uint current_major = 0;
        uint current_minor = 0;
        uint current_patch = 0;
        boost::smatch matches;
        boost::regex_match(current_version,matches,pattern);
        if(matches.size()==4){
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
            uint new_major,new_minor,new_patch;
            boost::smatch matches;
            boost::regex_match(version,matches,pattern);
            if(matches.size()==4){
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
        //! @todo Get the name and email of the user from ~/.stencila/.config
        std::string name = "";
        std::string email = "";
        // Set the path to "empty" (if already non-empty will leave unchanged, if not will be transient) 
        // to ensure component has a working directory
        path("");
        // Get, or create, repository for the component and tag it.
        Repository* repo = repo_(true);
        if(repo->head()=="<none>") STENCILA_THROW(Exception,"Component has not been commited. Please do a commit() before a version().");
        repo->tag(new_version,tag_message,name,email);
        return self();
    }

    /**
     * Provide a particular version of the component in it's `.version` subdirectory
     * 
     * @param  version [description]
     * @return         [description]
     */
    Class& provide(const std::string& version) const {
        // Check this is a valid version number 
        
        // Create the version's directory
        boost::filesystem::create_directories(path()+"/.versions/"+version);

        // Put the files for that version into the version's directory
        // See http://stackoverflow.com/a/10822596 which suggests something like this is necessary
        //      git_reference_name_to_oid() to retrieve the oid of the master branch
        //      git_commit_lookup() to retreive a commit from an oid
        //      git_commit_tree() to retrieve the tree of a commit
        //      git_iterator_for_tree() to recursively browse all the leafs of the tree (and its subtrees)

        return self();   
    }

    /**
     * Clone the component, creating a read only versionable clone
     * 
     * @return [description]
     */
    Class clone(){
        // Can only clone if has a repository
        return Class();
    }

    //fetch/pull/merge/sync

    /**
     * For the component, creating a copy that can be modified
     * 
     * @return [description]
     */
    Class fork(){
        Class fork = self();
        return fork;
    }

    /**
     * @}
     */
    

};

}
