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

/**
 * Base class for all Stencila components
 *
 */
class Component {
public:

    typedef Utilities::Git::Repository  Repository;
    typedef Utilities::Git::Commit      Commit;

    /**
     * Component classes
     *
     * Provides runtime information on Component classes
     * 
     * @{
     */

protected:

    /**
     * An enumeration of `Component` classes
     *
     * This `enum` serves to make it explicit that a variable or
     * function argument refers to a component class. Using an `int` or similar
     * could lead to ambiguities. An integer based code (as opposed to say a string based code)
     * makes it fast to lookup the type information (see `classes` below).
     * Having the `enum` values defined here, in one place, reduces the likelihood that 
     * the same integer code is given to more than one type. 
     */
    enum ClassCode {
        NoCode = 0,
        ComponentCode = 1,
        PackageCode = 2,
        StencilCode = 3,
        RContextCode = 4
    };

    /**
     * Number of `Class`s in the `classes` array.
     *
     * This number should be greater than the greatest integer
     * value in the `ClassCode` enumeration.
     */
    static const uint class_codes_ = 10;

    /**
     * The class' code, used for templated functions.
     * All derived classes MUST override this.
     */
    static const ClassCode class_ = ComponentCode;

public:

    /**
     * Structure repesenting a `Component` class.
     *
     * Serves as an entry in the `classes` array providing dynamic lookup
     * of type information, in particular, "virtual" functions.
     */
    struct Class {
        /**
         * Name of the class 
         */
        const char* name;

        /**
         * @name Page
         * 
         * Pageing function for class
         *
         * Used to serve a page for the component
         */
        typedef std::string (*Page)(const Component* component);
        Page page;

        /**
         * @name Message
         *
         * Messaging function for the class
         *
         * Used to send a message to a component when it is
         * being served
         */
        typedef std::string (*Message)(Component* component, const std::string& message);
        Message message;
    };

private:

    /**
     * Array of classes that have been defined
     *
     * See `component.cpp` for definitions of core classes
     */
    static Class classes_[class_codes_];

public:

    /**
     * Define a `Component` class
     *
     * This places an entry in `Component::classes_` so that information
     * on the class can be reteived later useing a `ClassCode`
     * 
     * @param code `ClassCode` for the class
     * @param clas `Class` object
     */
    static void define(ClassCode code, const Class& clas){
        classes_[code] = clas;
    }

    /**
     * @}
     */
    
   /**
     * Component instances: delaration, storage and reteival
     *
     * Provides for registration and retrieval of components from
     * both in memory and on disk.
     * 
     * @{
     */
    
public:

    /**
     * Structure representing a `Componet` instance.
     *
     * Holds a `ClassCode` and a pointer to the instance.
     */
    struct Instance {
        ClassCode code;
        Component* pointer;

        /**
         * Does this Instance refer to a component?
         *
         * Used to return "null" instances from methods below
         */
        operator bool(void){
            return pointer!=nullptr;
        }
    };

private:

    /**
     * A lookup table of instances keyed by component address
     *
     * Provides a registry of Component instances that
     * are in memory. Not all component's will be 
     * in this registry, they have to be registered first using
     * the `declare()` method
     */
    static std::map<std::string,Instance> instances_;

public:

   /**
     * Declare a component for retreival later
     *
     * The component is registed in `instances_` using its address.
     * If it does not yet have an address, one is created.
     *
     * This method MUST be overidden by derived classes, usually like this
     *
     *   <class> declare(void) {
     *       Component::declare(<class>Code);
     *       return *this;
     *   }
     * 
     * @param code     `ClassCode` for the class of component
     */
    Component& declare(ClassCode code = ComponentCode) {
        instances_[address(true)] = {code,this};
        return *this;
    }

    /**
     * Get a component instance with a given address
     *
     * This method is primarily used internally for retrieving
     * a component that is expected, but may not be, in memory.
     *
     * @param address Address of the component
     */
    static Instance retrieve(const std::string& address){
        auto i = instances_.find(address);
        if(i!=instances_.end()) return i->second;
        else return {NoCode,nullptr};
    }

    /**
     * Get the store paths
     */
    static std::vector<std::string> stores(void){
        return {
            Host::current_dir(),
            Host::user_dir(),
            Host::system_dir()
        };
    }

    /**
     * Get the path of a component from the stores
     */
    static std::string lookup(const std::string& address){
        std::vector<std::string> address_bits;
        boost::split(address_bits,address,boost::is_any_of("/"));
        auto address_base = address_bits[0];

        std::string url = "";
        for(std::string store : stores()){
            std::string store_path = store+"/"+address_base;
            // Does the address_base exist in this store?
            if(boost::filesystem::exists(store_path)){
                return store_path;
            }
        }

        return "";
    }
    
    /**
     * Get a component with a given address, and optionally, a version requirement
     *
     * A component that is found in one of the stores will be instantiated in memory.
     * Note that there is no garbage collection of these components at present.
     * 
     * @param address Address of component
     * @param version Version required
     * @param comparison Version requirement comparision (e.g. >=, ==, >)
     */
    template<class Class=Component>
    static Class& get(const std::string& address,const std::string& version="",const std::string& comparison="==") {
        Class* component;

        Instance instance = retrieve(address);
        if(instance){
            component = static_cast<Class*>(instance.pointer);
        }
        else {
            std::string path = lookup(address);
            if(path.length()>0){
                component = new Class;
                component->read(path);
                component->declare(Class::class_);
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

    /**
     * Call a "virtual" method of a component via an `Instance`
     *
     */
    // Two versions are implemented below one for const Component, the other non-const.
    // Although Supplied... seems uneccesary (why not just use Args...) template deduction
    // failed without it.
    template<
        typename Return,
        typename Class,
        typename... Args,
        typename... Supplied
    >
    static Return call(const Instance& instance, Return (* Class::* method)(Component* component, Args... args), Supplied... supplied){
        return (classes_[instance.code].*method)(instance.pointer,supplied...);
    }

    template<
        typename Return,
        typename Class,
        typename... Args,
        typename... Supplied
    >
    static Return call(const Instance& instance, Return (* Class::* method)(const Component* component, Args... args), Supplied... supplied){
        return (classes_[instance.code].*method)(instance.pointer,supplied...);
    }

    /**
     * Call a "virtual" method of a component via a component address
     */
    template<
        typename Return,
        typename Class,
        typename... Args
    >
    static Return call(const std::string& address, Return (* Class::* method)(const Component* component, Args... args), Args... args){
        return call(retrieve(address),method,args...);
    }

    /**
     * @}
     */
    
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
        return *this;

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
    Component& title(const std::string& title) {
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
    Component& description(const std::string& description) {
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
    Component& keywords(const std::vector<std::string>& keywords) {
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
    Component& authors(const std::vector<std::string>& authors) {
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
    
protected:

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
        boost::filesystem::path unique_path = stores()[1];
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
    Component& path(const std::string& path, bool force=false) {
        path_set_(path);
        return *this;
    }

    /**
     * Get the address of the component
     */
    std::string address(bool ensure = false) const {
        std::string path = path_get_(ensure);
        if(path.length()>0){
            std::string address = path;
            // Remove store prefix to obtain address
            for(auto store : stores()){
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
    Component& destroy(void){
        boost::filesystem::path path_full = path_get_();
        if(boost::filesystem::exists(path_full)){
            boost::filesystem::remove_all(path_full);
        }
        return *this;
    }

    /**
     * Create a file within the component's working directory
     * 
     * @param path Filesystem path within the working directory
     */
    Component& create(const std::string& path,const std::string& content="\n"){
        boost::filesystem::path path_full(path_get_(true));
        path_full /= path;
        if(!boost::filesystem::exists(path_full)){
            std::ofstream file(path_full.string());
            file<<content;
            file.close();
        }
        return *this;
    }

    /**
     * Delete a file within the component's working directory
     */
    Component& delete_(const std::string& path){
        boost::filesystem::path path_full(path_get_());
        path_full /= path;
        if(boost::filesystem::exists(path_full)){
            boost::filesystem::remove_all(path_full);
        }
        return *this;
    }

    /**
     * Read the component from a directory
     *
     * Note that reading a component from path and then reading it from a different
     * path will move the component directory to the new path.
     *
     * This method must be overidden by derived classes to implement
     * class specific read formats but call this base method so that `path` is set correctly.
     * 
     * @param from Filesystem path to component
     */
    Component& read(const std::string& from=""){
        path_set_(from);
        return *this;
    }
    
    /**
     * Write the Component to a directory
     *
     * This method must be overidden by derived classes to implement
     * class specific write formats but call this base method so that `path` is set correctly.
     * 
     * @param to Filesystem path to component
     */
    Component& write(const std::string& to=""){
        path_set_(to);
        return *this;
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

    /**
     * Commit the component
     *
     * This method should be overriden by derived classes so that
     * their `write` method is called before commiting
     */
    Component& commit(const std::string& message="") {
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
        return *this;
    }

    std::vector<Commit> history(void) const {
        Repository* repo = repo_();
        if(repo) return repo->history();
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

    Component& version(const std::string& version,const std::string& message="") {
        std::string new_version;
        std::string tag_message = message;
        std::string current_version = Component::version();

        boost::regex pattern("^(\\d+)\\.(\\d+)\\.(\\d+)$");

        auto regex_uint = [](const boost::smatch& matches,uint index){
            return boost::lexical_cast<uint>(std::string(matches[index].first,matches[index].second));
        };

        // Extract the semantic parts of the current version
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
        return *this;
    }

    /**
     * Provide a particular version of the component in it's `.version` subdirectory
     * 
     * @param  version [description]
     * @return         [description]
     */
    Component& provide(const std::string& version) {
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
            version_repo.checkout_tag(version);
            // Remove version .git directory
            boost::filesystem::remove_all(version_path+"/.git");
        }
        return *this;   
    }

    /**
     * @}
     */

    /**
     * @name Web interface
     *
     * Methods for serving components
     * 
     * @{
     */    
    
    /**
     * Serve this component
     * 
     * This method declares the component,
     * ensures that the `Server` is started and returns the
     * components URL.
     * 
     * @return URL for this component
     */
    std::string serve(ClassCode code);

    /**
     * View this component in the default web browser
     *
     * This method serves this componet and then opens it's address in the 
     * default browser.
     */
    void view(ClassCode code){
        // Serve this component so that it is available to be viewed via Server
        std::string url = serve(code);
        // Open URL in default browser
        #ifdef _WIN32 || _WIN64
           ShellExecute(NULL, "open", url, NULL, NULL, SW_SHOWNORMAL);
        #elif __APPLE__
            std::system(("open \""+url+"\"").c_str());
        #elif __linux
            // Open using xdg-open with all output redirected to null device
            std::system(("2>/dev/null 1>&2 xdg-open \""+url+"\"").c_str());
        #endif
    }

    /**
     * Generate a default home page for the server that is serving
     * components.
     */
    static std::string home(void){
        return R"(
            <html>
                <head>
                    <title>Stencila</title>
                </head>
            </html>
        )";
    }

    /**
     * Generate a page for a component at an address
     *
     * Currently, this uses `retrieve()` so will not get components
     * on disk. As such components need to be `declare()`d or `served()`d first 
     *
     * @param  address Address of component
     */
    static std::string page(const std::string& address){
        Instance instance = retrieve(address);
        if(not instance) return "<html><body>No component at address \""+address+"\"</body></html>";
        else return call(instance,&Class::page);
    }

    /**
     * Generate a web page for a component
     *
     * This method should be overriden by derived
     * classes and `define()`d in `classes_`.
     */
    static std::string page(const Component* component){
        return "";
    }

    /**
     * Process a message for the component at an address
     * 
     * @param  address Address of component
     * @param  message JSON request message
     * @return         JSON response message
     */
    static std::string message(const std::string& address,const std::string& message){
        Instance instance = retrieve(address);
        if(not instance) return "error: no component at address \""+address+"\"";
        else return call(instance,&Class::message,message);
    }

    /**
     * Process a message for a Component
     *
     * This method intentionally does nothing. It should be overriden by derived
     * classes and `define()`d in `classes_`.
     */
    static std::string message(Component* component, const std::string& message){
        return "";
    }

    /**
     * @}
     */

};

}
