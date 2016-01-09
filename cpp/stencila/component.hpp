#pragma once

#include <algorithm>
#include <string>
#include <vector>
#include <tuple>
#include <map>

#include <stencila/helpers.hpp>
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
	std::string address(void) const;

     /**
	 * Get this component's address, assigning
	 * an address if it does not yet have one
	 */
	std::string address(bool ensure);

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
	Component& create(const std::string& path, const std::string& content="\n");

	/**
	 * Read a file withing the component's working directory
	 * 
	 * @param  path    Filesystem path within the working directory
	 * @param  content String content to write
	 */
	Component& write_to(const std::string& path, const std::string& content);

	/**
	 * Read a file withing the component's working directory
	 * 
	 * @param  path    Filesystem path within the working directory
	 */
	std::string read_from(const std::string& path) const;

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
	 * @param path Filesystem path to component
	 */
	Component& read(const std::string& path="");
	
	/**
	 * Write the Component to a directory
	 *
	 * This method must be overidden by derived classes to implement
	 * class specific write formats but call this base method so that `path` is set correctly.
	 * 
	 * @param path Filesystem path to component
	 */
	Component& write(const std::string& path="");
	
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
	 * @return Path of the newly cloned component
	 */
	static std::string clone(const std::string& address);

	/**
	 * Fork a component
	 *
	 * @param  from Address of component to be forked
	 * @param  to Address of new component
	 * @return Path of the newly cloned component
	 */
	static std::string fork(const std::string& from, const std::string& to);

	/**
	 * Is this component managed?
	 */
	bool managed(void) const;

	/**
	 * Make this a managed component
	 */
	Component& managed(bool yes);

	/**
	 * Publish this component so that it is accessible to
	 * others
	 */
	Component& publish(const std::string& address);

	/**
	 * Get the origin for this component
	 * 
	 * @return  URL of the origin; empty string if this component is not a clone
	 */
	std::string origin(void) const;

	/**
	 * Sync the master branch with the origin
	 */
	Component& sync(void);

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
	 * Get current branch
	 */
	std::string branch(void) const;

	/**
	 * Switch to a branch
	 * 
	 * @param  branch Branch name
	 */
	Component& branch(const std::string& branch);

	/**
	 * Get a list of branches
	 */
	std::vector<std::string> branches(void) const;

	/**
	 * Create a new branch
	 * 
	 * @param  new_branch Name of the new branch
	 */
	Component& sprout(const std::string& new_branch, const std::string& from_branch = "master");

	/**
	 * Merge one branch into another branch
	 *
	 * @param from_branch Name of the branch to merge commits from
	 * @param into_branch Name of the branch to merge commits into
	 */
	Component& merge(const std::string& from_branch, const std::string& into_branch = "master");

	/**
	 * Delete a branch
	 * 
	 * @param  branch Branch name
	 */
	Component& lop(const std::string& branch);

	/**
	 * Provide a particular version or branch of the component
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
		NoneType,

		ComponentType,
		StencilType,
		ThemeType,
		SheetType,

		PythonContextType,
		
		RContextType,
		RSpreadType
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
		 * @name requesting
		 * 
		 * Requester function for class
		 *
		 * Used to respond to a web request on component 
		 */
		typedef std::string (*Requesting)(
			Component* component, const std::string& verb,
			const std::string& method, const std::string& body
		);
		Requesting requesting = nullptr;

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

		Class(const char* name, Pageing pageing, Requesting requesting, Calling calling):
			defined(true),
			name(name),
			pageing(pageing),
			requesting(requesting),
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
		Instance(void): type_(NoneType),pointer_(nullptr){};
		Instance(Type type, Component* pointer): type_(type),pointer_(pointer){};

		bool exists(void) const {
			return pointer_;
		}

		Type type(void) const {
			return type_;
		}

		Component* pointer(void) const {
			return pointer_;
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
	 * Instantiate a component
	 *
	 * A "callback" to a function in the host environment
	 * e.g. R, Python which instantiates a component as necessary
	 * for that environment (e.g. attaching a context)
	 */
	typedef Component* (*Instantiate) (const std::string& type, const std::string& path);
	static Instantiate instantiate;

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
	 * Is this component held?
	 */
	bool held(void) const;

	/**
	 * No longer hold this component in the list of instances
	 */
	Component& unhold(void);

	/**
	 * Get a list of all components held with address and type strings
	 */
	static std::vector<std::pair<std::string,std::string>> held_list(void);

	/**
	 * Get the type of a component at path
	 * 
	 * @param  path Filesystem path to component
	 */
	static Type type(const std::string& path);

	/**
	 * Get the name of the component type
	 * 
	 * @param  type
	 */
	static std::string type_name(const Type& type);

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
		Type arg(unsigned int index,const std::string& name="") const {
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
	Component& view(Type type);

	/**
	 * Create a preview image of this component
	 *
	 * @param  path       Path of the image to be generated
	 */
	Component& preview(Type type, const std::string& path);

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
	 * Respond to a web request to a component address
	 *
	 * Gets the component and dispatches it's the `requesting` method
	 *
	 * @param  component  A pointer to a stencil
	 * @param  verb       HTML verb (a.k.a. method) e.g. POST
	 * @param  method     Name of method requested
	 * @param  body       Request body (usually JSON)
	 */
	static std::string request_dispatch(
		const std::string& address,
		const std::string& verb,
		const std::string& method,
		const std::string& body
	);

	/**
	 * Exception for invalid requests (e.g wrong method name or HTTP method)
	 */
	class RequestInvalidException : public Exception {};

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
	 * Generate a default index page for the server that is serving
	 * components.
	 */
	static std::string index(void);

	/**
	 * Generate a extra constent for adding to component pages when served.
	 */
	static std::string extras(void);

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

/**
 * Generate a HTML document as the basis for a 
 */
template<class Type>
Html::Document Component_page_doc(const Type& component);

}
