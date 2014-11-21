#pragma once

#include <algorithm>
#include <string>
#include <vector>
#include <map>

#include <stencila/host.hpp>
#include <stencila/git.hpp>
#include <stencila/html.hpp>
#include <stencila/json.hpp>
#include <stencila/string.hpp>

namespace Stencila {

/**
 * Base class for all Stencila components
 *
 */
class Component {
public:

	Component(void):
		meta_(nullptr){
	}

	Component(const Component& other):
		meta_(nullptr){
	}

	Component(const std::string& address){
		initialise(address);
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
     * Initialise a component
     * 
     * @param  address Address of component
     */
    Component& initialise(const std::string& address);

	/**
	 * Get this component's path
	 *
	 * @param ensure Ensure a path is set if component does not yet have one?
	 */
	std::string path(bool ensure = false) const;

	/**
	 * Set this component's path
	 *
	 * If an empty string is supplied as `path` then a unique path under the "transient"
	 * subdirectory of the user's Stencila library will be created.
	 * 
	 * @param path Path to component
	 */
	Component& path(const std::string& path);

	/**
	 * Set this component's path
	 * 
	 * This overload prevents ambiguities with path(bool) when calling path("")
	 */
	Component& path(const char* path);

	/**
	 * Get this component's address
	 */
	std::string address(bool ensure = false);

	/**
	 * Get the filesystem paths of the Stencila stores
	 *
	 * `STENCILA_STORES` can be set as an environment variable.
	 * It serves the same function as [`PYTHONPATH` in Python](https://docs.python.org/2/using/cmdline.html#envvar-PYTHONPATH) 
	 * and [`R_LIBS` in R](http://stat.ethz.ch/R-manual/R-devel/library/base/html/libPaths.html)
	 */
	static std::vector<std::string> stores(void);

    /**
     * Locate a component in the stores
     * i.e. convert an `address` into a `path`
     *
     * This method is used by the `get()` method below but also by the `Server` class to 
     * statically serve a component file
     */
    static std::string locate(const std::string& address);

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
	 * Clone a component
	 *
	 * @param  address Address of component to be cloned
	 */
	static void clone(const std::string& address);

	/**
	 * Fork a component
	 *
	 * @param  from Address of component to be forked
	 * @param  to Address of new component
	 */
	static void fork(const std::string& from, const std::string& to);

	/**
	 * Get the origin for this component
	 * 
	 * @return  URL of the origin; empty string if this component is not a clone
	 */
	std::string origin(void) const;

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
	 * @name Instances
	 *
	 * Methods for the delaration, storage and reteival of components.
	 * Provides for registration and retrieval of components from
	 * both in memory and on disk.
	 *
	 * Methods implemented in `component-instance.cpp`
	 * 
	 * @{
	 */
	
	 class Call;

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
	enum Type {
		NoType,
		ComponentType,
		StencilType,
		ThemeType,

		PythonContextType,
		RContextType
	};

	/**
	 * Number of `Class`s in the `classes` array.
	 *
	 * This number should be greater than the greatest integer
	 * value in the `Type` enumeration.
	 */
	static const unsigned int types_ = 10;

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

	/**
	 * Define a `Component` class
	 *
	 * This places an entry in `Component::classes_` so that information
	 * on the class can be reteived later useing a `Type`
	 * 
	 * @param type `Type` for the class
	 * @param clas `Class` object
	 */
	static void class_(Type type, const Class& clas);

	/**
	 * Obtain a `Class` definition
	 *
	 * @param type `Type` for the class
	 */
	static const Class& class_(Type type);

	/**
	 * Definition of all core component classes
	 */
	static void classes(void);

	/**
	 * Structure representing a `Component` instance currently
	 * in memory.
	 */
	class Instance {
	public:
		Instance(void): type_(NoType),pointer_(nullptr){};
		Instance(Type type, Component* pointer): type_(type),pointer_(pointer){};

		bool exists(void) const {
			return pointer_;
		}

		Type type(void) const {
			return type_;
		}

		template<class Class=Component> 
		Class& as(void) {
			return *static_cast<Class*>(pointer_);
		}
	private:
		Type type_;
		Component* pointer_;
		friend class Component;
	};

	/**
	 * Declare a component for retreival later
	 *
	 * The component is registed in `instances_` using its address.
	 * If it does not yet have an address, one is created.
	 *
	 * This method MUST be overidden by derived classes, usually like this
	 *
	 *   <class> hold(void) {
	 *       Component::hold(<class>Type);
	 *       return *this;
	 *   }
	 * 
	 * @param type     `Type` for the class of component
	 */
	Component& hold(Type type = ComponentType);

    /**
     * Get the type of a component at path
     * 
     * @param  path Filesystem path to component
     */
    static Type type(const std::string& path);

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
	static Instance get(const std::string& address, const std::string& version="",const std::string& comparison="==");

	/** 
	 * @}
	 */

	/**
	 * @name Calling
	 *
	 * Methods for dynamically calling methods of a component
	 *
	 * Methods implemented in `component-call.cpp`
	 * 
	 * @{
	 */
	    
	struct Call {
	    std::string what_;
	    Json::Document args_;
	    Json::Document kwargs_;
	
	    Call(const std::string& what):
	        what_(what){
	    }
	
	    Call(const std::string& what, const std::string& args):
	        what_(what),
	        args_(args){
	    }
	
	    Call(const std::string& what, const std::string& args, const std::string& kwargs):
	        what_(what),
	        args_(args),
	        kwargs_(kwargs){
	    }
	
	    std::string what(void) const {
	        return what_;
	    }
	
	    unsigned int args(void) const {
	        return args_.size();
	    }
	
	    template<typename Type=std::string>
	    Type arg(int index,const std::string& name="") const {
	        // Get argument string
	        std::string arg;
	        if(name.length()>0){
	            if(kwargs_.has(name)) return kwargs_[name].as<Type>();
	            else STENCILA_THROW(Exception,"Argument \""+name+"\" not supplied");
	        }
	        else if(args_.size()<index+1){
	            STENCILA_THROW(Exception,"Not enough arguments supplied");
	        }
	        else return args_[index].as<Type>();
	    }
	};

	/**
	 * Call a "virtual" method of a component via an `Instance`
	 *
	 */
	// Two versions are implemented below one for const Component, the other non-const.
	// Although Supplied... seems uneccesary (why not just use Args...) template deduction
	// failed without it.
	template<typename Return,typename Class,typename... Args,typename... Supplied>
	static Return call(const Instance& instance, Return (* Class::* method)(Component* component, Args... args), Supplied... supplied){
		return (class_(instance.type_).*method)(instance.pointer_,supplied...);
	}

	template<typename Return,typename Class,typename... Args,typename... Supplied>
	static Return call(const Instance& instance, Return (* Class::* method)(const Component* component, Args... args), Supplied... supplied){
		return (class_(instance.type_).*method)(instance.pointer_,supplied...);
	}

	/**
	 * Call a "virtual" method of a component via a component address
	 */
	template<typename Return,typename Class,typename... Args>
	static Return call(const std::string& address, Return (* Class::* method)(const Component* component, Args... args), Args... args){
		return call(get(address).as<Class>(),method,args...);
	}

	std::string call(const Call& call);

	/**
	 * @}
	 */
	

	/**
	 * @name Serving
	 *
	 * Methods for serving a component over a network.
	 * These methods often need to be overrided by derived
	 * component classes.
	 *
	 * Methods implemented in `component-serve.cpp`
	 * 
	 * @{
	 */   
	
	/**
	 * Serve this component
	 * 
	 * This method declares this component, ensures that the `Server` is started and returns this
	 * component's URL.
	 * 
	 * @return URL for this component
	 */
	std::string serve(Type type);

	/**
	 * View this component in the default web browser
	 *
	 * This method serves this componet and then opens it's address in the 
	 * default browser on the host machine.
	 */
	void view(Type type);

	/**
	 * Generate a page for a component at an address
	 *
	 * Currently, this uses `retrieve()` so will not get components
	 * on disk. As such components need to be `declare()`d or `served()`d first 
	 *
	 * @param  address Address of component
	 */
	static std::string page(const std::string& address);

	/**
	 * Generate a HTML page for a component
	 *
	 * This method should be overriden by derived
	 * classes and `define()`d in `classes_`.
	 */
	static std::string page(const Component* component);

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
	static std::string page(const Component* component,const std::string& title,const std::string& theme);

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
	static std::string message(const std::string& address,const std::string& message);

	/**
	 * Process a message for a Component
	 *
	 * This method intentionally does nothing. It should be overriden by derived
	 * classes and the overriding method `define()`d.
	 */
	static std::string message(Component* component, const std::string& message);

    /**
     * Generate a default home page for the server that is serving
     * components.
     */
    static std::string home(void);

	/**
	 * @}
	 */

protected:

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

	/**
	 * Array of classes that have been defined
	 *
	 * See `component.cpp` for definitions of core classes
	 */
	static Class classes_[types_];


	/**
	 * A lookup table of Component instances keyed by component address
	 *
	 * Provides a registry of Component instances that
	 * are in memory. Not all component's will be 
	 * in this registry, they have to be registered first using
	 * the `declare()` method
	 */
	static std::map<std::string,Instance> instances_;

};

}
