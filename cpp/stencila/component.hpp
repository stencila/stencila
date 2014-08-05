#pragma once

#include <algorithm>
#include <string>
#include <vector>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>
#include <boost/regex.hpp>

#include <stencila/host.hpp>
#include <stencila/git.hpp>
#include <stencila/html.hpp>
#include <stencila/json.hpp>

namespace Stencila {

/**
 * Base class for all Stencila components
 *
 */
class Component {
public:

    typedef Git::Repository  Repository;
    typedef Git::Commit      Commit;

    /**
     * Component classes
     *
     * Provides runtime information on Component classes
     * 
     * @{
     */
        
    struct Call {
        std::string what_;
        std::vector<std::string> args_;
        std::map<std::string,std::string> kwargs_;

        Call(const std::string& what):
            what_(what){
        }

        Call(const std::string& what, const std::vector<std::string>& args):
            what_(what),
            args_(args){
        }

        Call(const std::string& what, const std::vector<std::string>& args, const std::map<std::string,std::string>& kwargs):
            what_(what),
            args_(args),
            kwargs_(kwargs){
        }

        std::string what(void) const {
            return what_;
        }

        uint args(void) const {
            return args_.size();
        }

        template<typename Type=std::string>
        Type arg(uint index,const std::string& name="") const {
            if(name.length()>0){
                auto i = kwargs_.find(name);
                if(i!=kwargs_.end()) return boost::lexical_cast<Type>(i->second);
                else STENCILA_THROW(Exception,"Argument \""+name+"\" not supplied");
            }
            if(args_.size()<index+1){
                STENCILA_THROW(Exception,"Not enough arguments supplied");
            }
            return boost::lexical_cast<Type>(args_[index]);
        }

    };

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

        PythonContextCode = 4,
        RContextCode = 5
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
         * Flag to indicate that the class has been defined
         */
        bool defined = false;

        /**
         * Name of the class 
         */
        const char* name = "";

        /**
         * @name pageing
         * 
         * Pageing function for class
         *
         * Used to serve a page for the component
         */
        typedef std::string (*Pageing)(const Component* component);
        Pageing pageing = nullptr;

        /**
         * @name calling
         *
         * Calling function for the class
         *
         * Used to call a method for the component (usually remotely)
         */
        typedef std::string (*Calling)(Component* component, const Call& call);
        Calling calling = nullptr;

        
        Class(void):
            defined(false){
        }

        Class(const char* name, Pageing pageing, Calling calling):
            defined(true),
            name(name),
            pageing(pageing),
            calling(calling){
        }
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
     * Obtain a `Class` definition
     *
     * @param code `ClassCode` for the class
     */
    static const Class& definition(ClassCode code){
        const Class& clas = classes_[code];
        if(not clas.defined) STENCILA_THROW(Exception,str(boost::format("Class with code <%s> has not been defined")%code));
        return clas;
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
        operator bool(void) const {
            // When pointer is nullptr it evaluates to false
            return pointer;
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
     * a component that is expected to, but may not, be in memory.
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
     *
     * `STENCILA_STORES` can be set as an environment variable.
     * It serves the same function as [`PYTHONPATH` in Python](https://docs.python.org/2/using/cmdline.html#envvar-PYTHONPATH) 
     * and [`R_LIBS` in R](http://stat.ethz.ch/R-manual/R-devel/library/base/html/libPaths.html)
     */
    static std::vector<std::string> stores(void){
        std::vector<std::string> stores = {
            Host::current_dir()
        };
        const char* more = std::getenv("STENCILA_STORES");
        if(more) {
            std::vector<std::string> more_stores;
            boost::split(more_stores,more,boost::is_any_of(";"));
            for(std::string store : more_stores) stores.push_back(store);
        }
        stores.push_back(Host::user_dir());
        stores.push_back(Host::system_dir());
        return stores;
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

    static std::string resolve(const std::string& path){
        std::string url = "";
        for(std::string store : stores()){
            std::string store_path = store+"/"+path;
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
        return (definition(instance.code).*method)(instance.pointer,supplied...);
    }

    template<
        typename Return,
        typename Class,
        typename... Args,
        typename... Supplied
    >
    static Return call(const Instance& instance, Return (* Class::* method)(const Component* component, Args... args), Supplied... supplied){
        return (definition(instance.code).*method)(instance.pointer,supplied...);
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
        unique_path /= boost::filesystem::unique_path("temp/%%%%-%%%%-%%%%-%%%%");
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
                // and create the path if necessary
                if(not boost::filesystem::exists(new_path)){
                    boost::filesystem::create_directories(new_path);
                }
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
            // Remove store prefix
            for(auto store : stores()){
                if(address.substr(0,store.length())==store){
                    address = address.substr(store.length()+1);
                }
            }
            // Remove file names
            // @fixme this is a temporary hack
            boost::replace_last(address,"/stencil.html","");
            return address;
        }
        else return "";
    }

    /**
     * List files and folders in a components directory 
     */
    struct File {
        std::string name;
        std::string type;

        static bool by_name(const File& a, const File& b){
            return a.name<b.name;
        }
    };
    std::vector<File> list(const std::string& subdirectory=""){
        using namespace boost::filesystem;
        std::vector<File> files;
        std::string dir = boost::filesystem::path(path()).parent_path().string() + subdirectory;
        if(exists(dir) and is_directory(dir)){
            directory_iterator end ;
            for(directory_iterator iter(dir) ; iter != end ; ++iter){
                File file;
                file.name = iter->path().filename().string();
                if(is_regular_file(iter->status())) file.type = "f";
                else if(is_directory(iter->status())) file.type = "d";
                else file.type = "o";
                files.push_back(file);
            }
        }
        // Sort alphabetically
        std::sort(files.begin(),files.end(),File::by_name);
        return files;
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
                catch(Git::GitNoRepoError){
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
        #if defined(_WIN32) || defined(_WIN64)
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
        Html::Document page(R"(
            <html>
                <head>
                    <title>Stencila</title>
                </head>
                <body></body>
            </html>
        )");
        auto list = page.one("body").append("ul");
        for(auto instance : instances_){
            list.append("li",instance.first);
        }
        return page.dump();
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
        if(not instance) return "<html><head><title>Error</title></head><body>No component at address \""+address+"\"</body></html>";
        else return call(instance,&Class::pageing);
    }

    /**
     * Generate a HTML page for a component
     *
     * This method should be overriden by derived
     * classes and `define()`d in `classes_`.
     */
    static std::string page(const Component* component){
        return "";
    }

    /**
     * Generate a HTML page for a component with a title and theme
     *
     * This function is provided for the convienience of derived classes: in their
     * `static std::string page(const Component*)` overrides they can call this to
     * generate a standard page which their themes may then augment
     * 
     * @param  component [description]
     * @param  title     [description]
     * @param  theme     [description]
     * @return           [description]
     */
    static std::string page(const Component* component,const std::string& title,const std::string& theme) {
        using boost::format;
        return str(format(R"(
            <html>
                <head>
                    <title>%s</title>
                    <link rel="stylesheet" type="text/css" href="/%s/theme.css" />
                </head>
                <body>
                    <script src="/core/themes/boot.js"></script>
                    <script src="/%s/theme.js"></script>
                </body>
            </html>
        )") % title % theme % theme);
    }

    /**
     * Process a message for the component at an address
     *
     * We use [WAMP](http://wamp.ws/) as the message protocol.
     * Curently that is only partially implemented.
     * 
     * @param  address Address of component
     * @param  message A WAMP message
     * @return         A WAMP message
     */
    static std::string message(const std::string& address,const std::string& message){
        using boost::format;
        using Json::size;
        using Json::as;

        //WAMP basic spec is at https://github.com/tavendo/WAMP/blob/master/spec/basic.md
        
        // WAMP message codes used below.
        // From https://github.com/tavendo/WAMP/blob/master/spec/basic.md#message-codes-and-direction
        //static const int ERROR = 8;
        static const int CALL = 48;
        static const int RESULT = 50;
        //static const int YIELD = 70;

        //[ERROR, REQUEST.Type|int, REQUEST.Request|id, Details|dict, Error|uri]
        //[ERROR, REQUEST.Type|int, REQUEST.Request|id, Details|dict, Error|uri, Arguments|list]
        //[ERROR, REQUEST.Type|int, REQUEST.Request|id, Details|dict, Error|uri, Arguments|list, ArgumentsKw|dict]

        try {
            Instance instance = retrieve(address);
            if(not instance){
                return "[8, 0, 0, {}, \"no component at address\", [\"" + address + "\"]]";
            } else {

                Json::Document request(message);

                int items = size(request);
                if(items<1) STENCILA_THROW(Exception,"Malformed message");

                char code = as<int>(request[0]);
                if(code==CALL){
                    //[CALL, Request|id, Options|dict, Procedure|uri]
                    //[CALL, Request|id, Options|dict, Procedure|uri, Arguments|list]
                    //[CALL, Request|id, Options|dict, Procedure|uri, Arguments|list, ArgumentsKw|dict]
                    
                    if(items<2) STENCILA_THROW(Exception,"Malformed message");
                    int id = as<int>(request[1]);
                    
                    if(items<4) STENCILA_THROW(Exception,"Malformed message");
                    std::string procedure = as<std::string>(request[3]);

                    std::vector<std::string> args;
                    if(items>=5){
                        Json::Value& args_value = request[4];
                        args.resize(size(args_value));
                        for(uint i=0;i<args.size();i++) args[i] = as<std::string>(args_value[i]);
                    }

                    std::map<std::string,std::string> kwargs;
                    if(items>=6){
                        /**
                         * @fixme Not implemented
                         */
                        #if 0
                        Json::Value& kwargs_value = request[5];
                        for(int i=0;i<size(kwargs_value);i++){
                            auto value = kwargs_value[i];
                            auto name = 
                            args[name] = value;
                        }
                        #endif
                    }
                    
                    std::string result;
                    try {
                        Call call(procedure,args,kwargs);
                        result = Component::call(instance,&Class::calling,call);
                    }
                    catch(const std::exception& e){
                        return str(format("[8, 48, %d, {}, \"%s\"]")%id%e.what());
                    }
                    catch(...){
                        return str(format("[8, 48, %d, {}, \"unknown exception\"]")%id);         
                    }

                    //[RESULT, CALL.Request|id, Details|dict]
                    //[RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
                    //[RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list, YIELD.ArgumentsKw|dict]
                    Json::Document response;
                    response.type<Json::Array>()
                            .push(RESULT)
                            .push(id)                                // CALL.Request|id
                            .push(Json::Object())                          // Details|dict
                            .push(std::vector<std::string>{result}); // YIELD.Arguments|list
                    return response.dump();
                }
                return "[8, 0 , 0,{},\"unhandle message code\"]";
            }
        }
        // Most exceptions should be caught above and WAMP ERROR messages appropriate to the 
        // request type returned. The following are failovers if that does not happen...
        catch(const std::exception& e){
            return std::string("[8, 0, 0, {}, \"") + e.what() + "\"]";
        }
        catch(...){
            return "[8, 0, 0, {}, \"unknown exception\"]";         
        }
        // This exception is intended to capture errors in coding above where none of the branches
        // return a string
        STENCILA_THROW(Exception,"Implementation error; message not handles properly");
    }

    /**
     * Process a message for a Component
     *
     * This method intentionally does nothing. It should be overriden by derived
     * classes and `define()`d in `classes_`.
     */
    static std::string message(Component* component, const std::string& message){
        return "{}";
    }

    std::string call(const Call& call) {
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
        else if(what=="history():array"){
            Json::Document log;
            log.type<Json::Array>();
            for(auto commit : history()) {
                log.push(commit.name+" "+commit.email+" "+commit.message);
            }
            return log.dump();
        }

        else {
            STENCILA_THROW(Exception,"Method signature not registered for calling: "+what);
        }
        return "";
    }        

    /**
     * @}
     */

};

}
