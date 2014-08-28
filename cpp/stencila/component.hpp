#pragma once

#include <algorithm>
#include <string>
#include <vector>
#include <map>

#include <boost/format.hpp>
#include <boost/lexical_cast.hpp>

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
    static std::string locate(const std::string& address){
        for(std::string store : stores()){
            boost::filesystem::path path = boost::filesystem::path(store)/address;
            if(boost::filesystem::exists(path)) return path.string();
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
            std::string path = locate(address);
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
     * @name Input and output
     *
     * Methods implemented in `component-io.cpp`
     * 
     * @{
     */

    /**
     * Get the component's path
     *
     * @param ensure Ensure a path is set if component does not yet have one?
     */
    std::string path(bool ensure = false) const;

    /**
     * Set the component's path
     *
     * If an empty string is supplied as `path` then a unique path under the "transient"
     * subdirectory of the user's Stencila library will be created.
     * 
     * @param path Path to component
     */
    Component& path(const std::string& path);

    /**
     * Set the component's path
     * 
     * This overload prevents ambiguities with path(bool) when calling path("")
     */
    Component& path(const char* path);

    /**
     * Get the address of the component
     */
    std::string address(bool ensure = false);

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
    std::vector<File> list(const std::string& subdirectory="");

    /**
     * Destroy the component's entire working directory
     */
    Component& destroy(void);

    /**
     * Create a file within the component's working directory
     * 
     * @param path Filesystem path within the working directory
     */
    Component& create(const std::string& path,const std::string& content="\n");

    Component& write(const std::string& path,const std::string& content);

    /**
     * Delete a file within the component's working directory
     */
    Component& delete_(const std::string& path);

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
    Component& read(const std::string& from="");
    
    /**
     * Write the Component to a directory
     *
     * This method must be overidden by derived classes to implement
     * class specific write formats but call this base method so that `path` is set correctly.
     * 
     * @param to Filesystem path to component
     */
    Component& write(const std::string& to="");
    
    /**
     * @}
     */

    /**
     * @name Repository
     *
     * Methods implemented in `stencil-repo.cpp`
     * 
     * @{
     */   
    
    typedef Git::Repository  Repository;
    typedef Git::Commit      Commit;

    /**
     * Get a pointer to this component's repository
     *
     * @param ensure If the repository is not yet created should it be created?
     */
    Repository* repo(bool ensure = false) const;

    /**
     * Commit this component
     *
     * This method should be overriden by derived classes so that
     * their `write` method is called before commiting
     *
     * @param message Message to associate with the commit
     */
    Component& commit(const std::string& message="");

    /**
     * Get a list of commits made to this component
     */
    std::vector<Commit> commits(void) const;

    /**
     * Create a version for this component
     * 
     * @param  version Semantic version number (e.g. 0.3.2)
     * @param  message Message to associate with the version
     */
    Component& version(const std::string& version,const std::string& message="");

    /**
     * Get this component's current version
     */
    std::string version(void) const;

    /**
     * Get a list of versions available for this component
     */
    std::vector<std::string> versions(void) const;


    /**
     * Provide a particular version of the component
     * 
     * @param  version Version to provide
     */
    Component& provide(const std::string& version);

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
        auto list = page.select("body").append("ul");
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
        else if(what=="commits():array"){
            Json::Document log;
            log.type<Json::Array>();
            for(auto commit : commits()) {
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

private:

    /**
     * Component meta data
     *
     * Encapsulated as a separate class to reduce the minimum size of a 
     * Component object to the sizeof(Meta*). 
     */
    class Meta {
    public:
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

};

}
