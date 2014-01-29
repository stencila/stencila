#pragma once

#include <algorithm>
#include <string>
#include <vector>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/regex.hpp>

#include <stencila/host.hpp>
#include <stencila/utilities/git.hpp>

namespace Stencila {

template<class Class=void> class Component;

/**
 * Base class for all components that deals with cacheing and storage
 *
 * Only has protected static methods. Public interface is by functions in 
 * Stencila namespace.
 */
template<>
class Component<void> {

    /**
     * @name Cacheing
     * @{
     */
    
private:

    struct Pointer {
        std::string type;
        Component<void>* pointer;
    };
    static std::map<std::string,Pointer> cache_map_;
    
protected:

    /**
     * Cache a component for retreival later
     * 
     * @param type     String representing type of component
     * @param instance Component pointer
     */
    static void cache_(const std::string& address,const std::string& type,Component<void>* instance){
        cache_map_[address] = {type,instance};
    }

    /**
     * Get a component form the cache if it there
     *
     * @param address Address of the components
     * @param type Type (class name) of the component
     */
    static Component<void>* cached_(const std::string& address,const std::string& type=""){
        auto i = cache_map_.find(address);
        if(i!=cache_map_.end()){
            if(type.length()>0){
                if(i->second.type==type) return i->second.pointer;
            } else {
                return i->second.pointer;
            }
        }
        return 0;
    }

    /**
     * Remove a component from the cache
     * 
     * @param address Address of the component to remove
     */
    static void uncache_(const std::string& address){
        cache_map_.erase(address);
    }

    /**
     * @}
     */
    
    /**
     * @name Storage
     * @{
     */

protected:

    /**
     * Return the path user store
     */
    static std::string stores_user_(void){
        return Host::user_dir();
    }

    /**
     * Get the store paths
     */
    static std::vector<std::string> stores_(bool ensure=true){
        return {
            Host::current_dir(),
            Host::user_dir(),
            Host::system_dir()
        };
    }

    /**
     * Get the path of a component from the stores
     */
    static std::string stored_(const std::string& address){
        // Get stores
        std::vector<std::string> stores = stores_();

        std::vector<std::string> address_bits;
        boost::split(address_bits,address,boost::is_any_of("/"));
        auto address_base = address_bits[0];

        std::string url = "";
        for(std::string store : stores){
            std::string store_path = store+"/"+address_base;
            // Does the address_base exist in this store?
            if(boost::filesystem::exists(store_path)){
                return store_path;
            }
        }

        return "";
    }

    /**
     * @}
     */
    
    template<class Class>
    friend Class& obtain(const std::string&,const std::string&,const std::string&);

};
std::map<std::string,Component<void>::Pointer> Component<void>::cache_map_;

/**
 * Get a component with a given address, and optionally, a version requirement
 * 
 * @param address Address of component
 * @param version Version required
 * @param comparison Version requirement comparision (e.g. >=, ==, >)
 *
 * @todo Need to delete a newly obtained component
 * @todo Check version comparison
 */
template<class Class>
static Class& obtain(const std::string& address,const std::string& version,const std::string& comparison){
    Class* component;

    Component<void>* cached = Component<void>::cached_(address);
    if(cached){
        component = static_cast<Class*>(cached);
    }
    else {
        std::string path = Component<void>::stored_(address);
        if(path.length()>0){
            component = new Class;
            component->read(path);
        } else {
            STENCILA_THROW(Exception,"Component with address not found: "+address);
        }
    }

    // Provide required version number
    if(version.length()>0){
        if(comparison.length()==0 or comparison=="=="){
            component->provide(version);
        } else {
            STENCILA_THROW(Exception,"Version comparison operator not yet supported: "+comparison);
        }
    }

    return *component;
}
// Default argument values for this static template function do not
// compile for some reason so simulate them here.
template<class Class>
static Class& obtain(const std::string& address,const std::string& version){
    return obtain<Class>(address,version,"==");
}
template<class Class>
static Class& obtain(const std::string& address){
    return obtain<Class>(address,"","==");
}

/**
 * Component
 */
template<class Class>
class Component : public Component<void> {
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

    const Class& self(void) const {
        return static_cast<const Class&>(*this);
    }

    const char* type(void) const {
        return self().type_();
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
    
private:

    /**
     * Get the component's path and, optionally, ensure
     * it exists
     */
    const std::string& path_get_(bool ensure = false) const {
        static const std::string none = "";
        if(meta_){
            if(ensure and meta_->path.length()==0){
                path_set_unique_();
            }
            return meta_->path;
        } else {
            if(ensure){
                meta_ = new Meta;
                return path_get_(true);
            }
            else return none;
        }
    }

    void path_set_unique_(void) const {
        boost::filesystem::path unique_path = stores_user_();
        unique_path /= boost::filesystem::unique_path("~%%%%-%%%%-%%%%-%%%%");
        boost::filesystem::create_directories(unique_path);
        if(not meta_) meta_ = new Meta;
        meta_->path = unique_path.string();
    }

    void path_set_(const std::string& path){
        if(not meta_) meta_ = new Meta;
        std::string current_path = meta_->path;
        std::string new_path = path;
        // If the current path is empty...
        if(current_path.length()==0){
            // If the new path is empty then...
            if(new_path.length()==0){
                // call the private path_set_unique_ method
                path_set_unique_();
            } else {
                // and create the path.
                boost::filesystem::create_directories(new_path);
                meta_->path = new_path;
            }
        } 
        // If the current path is not empty...
        else {
            /// and the new path is not empty...
            if(new_path.length()>0){
                // create necessary directories for the following rename operation
                boost::filesystem::create_directories(new_path);
                // move (i.e rename) existing path to the new path.
                boost::filesystem::rename(current_path,new_path);
                meta_->path = new_path;
            }
        }
        
        cache_(address(),type(),this);
    }
    
public:

    /**
     * Get the component's path
     */
    const std::string& path(void) const {
        static const std::string none = "";
        if(not meta_) return none;
        else return meta_->path;
    }

    /**
     * Set the component's path
     *
     * If an empty string is supplied as `path` then a unique path under the "transient"
     * subdirectory of the user's Stencila library will be created.
     * 
     * @param path Path to component
     * @param force Force change of path if path already exists?
     *
     * @todo Implement `force` option
     */
    Class& path(const std::string& path, bool force=false) {
        path_set_(path);
        return self();
    }

    /**
     * Get the address of the component
     */
    std::string address(void) const {
        std::string path = path_get_();
        if(path.length()>0){
            std::string address = path;
            // Remove store prefix to obtain address
            for(auto store : stores_()){
                if(address.substr(0,store.length())==store){
                    return address.substr(store.length()+1);
                }
            }
            return address;
        }
        else return "";
    }

    /**
     * Destroy the component's entire working directory
     */
    Class& destroy(void){
        boost::filesystem::path path_full = path_get_();
        if(boost::filesystem::exists(path_full)){
            boost::filesystem::remove_all(path_full);
        }
        return self();
    }

public:

    /**
     * Create a file within the component's working directory
     * 
     * @param path Filesystem path within the working directory
     */
    Class& create(const std::string& path,const std::string& content="\n"){
        boost::filesystem::path path_full(path_get_(true));
        path_full /= path;
        if(!boost::filesystem::exists(path_full)){
            std::ofstream file(path_full.string());
            file<<content;
            file.close();
        }
        return self();
    }

    /**
     * Delete a file within the component's working directory
     */
    Class& delete_(const std::string& path){
        boost::filesystem::path path_full(path_get_());
        path_full /= path;
        if(boost::filesystem::exists(path_full)){
            boost::filesystem::remove_all(path_full);
        }
        return self();
    }

    /**
     * Read the component from a directory
     *
     * Note that reading a component from path and then reading it from a different
     * path will move the component directory to the new path
     * 
     * @param from Filesystem path to component
     */
    Class& read(const std::string& from=""){
        path_set_(from);
        self().read_();
        return self();
    }
    
    /**
     * Write the Component to a directory
     * 
     * @param to Filesystem path to component
     */
    Class& write(const std::string& to=""){
        path_set_(to);
        self().write_();
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

    /**
     * Get, and optionally create if it does not exist, the
     * component's repository
     */
    Repository* repo_(bool ensure = false) const {
        if(meta_){
            if(ensure and not meta_->repo){
                std::string path = path_get_();
                Repository* repo = new Repository;
                try{
                    repo->open(path);
                }
                catch(Utilities::Git::GitNoRepoError){
                    repo->init(path);
                }
                meta_->repo = repo;
            }
            return meta_->repo;
        } else {
            if(ensure){
                meta_ = new Meta;
                return repo_(true);
            }
            else return nullptr;
        }
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

    std::vector<std::string> versions(void) const {
        Repository* repo = repo_();
        if(repo){
            std::vector<std::string> versions = repo->tags();
            return versions;
        }
        else return std::vector<std::string>(0);
    }

    std::string version(void) const {
        Repository* repo = repo_();
        if(repo){
            std::string version = repo->tag();
            if(version.length()==0) version = "";
            return version;
        }
        else return "";
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
            uint new_major,new_minor,new_patch;
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
    Class& provide(const std::string& version) {
        // Check if the version already exists
        std::string version_path = path()+"/."+version;
        if(not boost::filesystem::exists(version_path)){
            // Check this is a valid version number 
            std::vector<std::string> vs = versions();
            if(std::count(vs.begin(),vs.end(),version)==0){
                STENCILA_THROW(Exception,"Component does not have version: "+version);
            }
            // Checkout the the version into version_path
            // Clone this repo into a version_path
            Repository version_repo;
            version_repo.clone(path(),version_path);      
            // Checkout version
            version_repo.checkout_tag(version);
            // Remove version .git directory
            boost::filesystem::remove_all(version_path+"/.git");
        }
        return self();   
    }

    /**
     * @}
     */
    
};

}
