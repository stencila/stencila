#pragma once

#include <algorithm>
#include <memory>
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
#include <stencila/wamp.hpp>

namespace Stencila {

/**
 * Base class for all Stencila components
 *
 */
class Component {
public:

	/**
	 * @name Construction and destruction
	 *
	 * Methods implemented in `component.cpp`
	 * 
	 * @{
	 */

	Component(void);

	Component(const Component& other);

	Component(const std::string& address);

	~Component(void);

	/**
	 * @}
	 */
	

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
	* Set this component's address
	*/
	Component& address(const std::string& address);

	/**
	* Set this component's address
	*
	* This overload prevents ambiguities with path(bool) when calling path("")
	*/
	Component& address(const char* address);

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
	Component& delete_file(const std::string& path);

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
	 * Clean up output files by deleting the `out` directory, if any, in 
	 * this comonent's path
	 */
	Component& vacuum(void);

	/**
	 * Destroy the component's entire working directory
	 */
	Component& destroy(void);

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
	std::string commit(const std::string& message="");

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
	 * @name Snapshot
	 *
	 * Methods implemented in `component-snapshots.cpp`
	 * 
	 * @{
	 */
	
	/**
	 * Take a snapshot of this component
	 */
	Component& store(void);

	/** 
	 * Restore this component from the last available snapshot
	 */
	Component& restore(void);

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
		FunctionType,

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
	 * Convert a type to it's name string
	 */
	static std::string type_to_string(const Type& type);

	/**
	 * Convert a type name string to a type
	 */
	static Type type_from_string(std::string string);

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

		std::string type_name(void) const {
			return Component::type_to_string(type_);
		}

		Component* pointer(void) const {
			return pointer_;
		}

		template<class Type> 
		Type as(void) const {
			return static_cast<Type>(pointer_);
		}

	private:
		Type type_;
		Component* pointer_;
	};

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
		 * The method for generating a HTML page
		 */
		typedef std::string (*PageMethod)(const Instance& instance);
		PageMethod page_method = nullptr;

		/**
		 * @name requesting
		 * 
		 * The method for handling HTTP requests
		 */
		typedef std::string (*RequestMethod)(
			const Instance& instance, const std::string& verb,
			const std::string& method, const std::string& body
		);
		RequestMethod request_method = nullptr;

		/**
		 * @name message_method
		 * 
		 * The method for handling WAMP messages
		 */
		typedef Wamp::Message (*MessageMethod)(
			const Instance& instance, const Wamp::Message& message
		);
		MessageMethod message_method = nullptr;


		Class(void):
			defined(false){
		}

		Class(const char* name, PageMethod page_method = nullptr, RequestMethod request_method = nullptr, MessageMethod message_method = nullptr):
			defined(true),
			name(name),
			page_method(page_method),
			request_method(request_method),
			message_method(message_method){
		}

		/**
		 * Set a `Class` for a `Type`
		 *
		 * This places an entry in `Component::classes_` so that information
		 * on the class can be reteived later useing a `Type`
		 * 
		 * @param type `Type` for the class
		 * @param clas `Class` object
		 */
		static void set(Type type, const Class& clas);

		/**
		 * Get a `Class` for a `Type`
		 *
		 * @param type `Type` for the class
		 */
		static const Class& get(Type type);

	 private:

		/**
		 * Array of classes that have been defined
		 */
		static Class classes_[types_];
	};

	/**
	 * Definition of all core component classes
	 */
	static void classes(void);

	/**
	 * Construct a component from string content
	 */
	static Component* create(const std::string& type, const std::string& content, const std::string& format = "json");

	/**
	 * Instantiate a component
	 *
	 * A "callback" to a function in the host environment
	 * e.g. R, Python which instantiates a component as necessary
	 * for that environment (e.g. attaching a context)
	 */
	typedef Component* (*Instantiate) (const std::string& type, const std::string& content, const std::string& format);
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
	 * Open a component from a path
	 *
	 * Includes checking whether the component can be restored
	 * from a previous snapshot.
	 */
	template<class Class>
	static Component* open(Type type, const std::string& path="");

	/**
	 * Save a component
	 *
	 * Writes component to disk as well as storing a snapshot if
	 * appropriate.
	 */
	Component& save(void);

	/**
	 * Close a component
	 * 
	 * Similar to save but also 
	 */
	Component& close(void);

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
	static std::string page_dispatch(const std::string& address);

	/**
	 * Respond to a web request to a component address
	 *
	 * Gets the component and dispatches to it's `request` method
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
	 * Respond to a websocket message to a component
	 *
	 * Resolves the component and dispatches to it's `message` method
	 *
	 * @param  message    Message text
	 * @param  connection Connection id
	 */
	static std::string message_dispatch(const std::string& message, int connection = 0);

	/**
	 * Exception for when a dispathced method is not defined for a class
	 */
	class MethodUndefinedException : public Exception {
	 public:
	 	MethodUndefinedException(const std::string& name, const Instance& instance, const char* file=0, int line=0):
	 		Exception(
	 			std::string("Dynamic method has not been defined for component class.") + 
					"\n  method: " + name +
					"\n  class: " + instance.type_name(),
				file,
				line
			){}
	};

	/**
	 * Default handler methods. These handle a request made to an instance.
	 * Usually just be casting the instance to the correct type and executing its realted
	 * method. See default implementations below. May be overidden by derived classes
	 */
	
	template<class Class>
	static std::string page_handler(const Instance& instance);

	template<class Class>
	static std::string request_handler(
		const Instance& instance,		
		const std::string& verb,
		const std::string& method,
		const std::string& body
	);

	template<class Class>
	static Wamp::Message message_handler(const Instance& instance, const Wamp::Message& message);

	/**
	 * Default implementation method
	 */

	std::string page(void);

	std::string request(const std::string& verb, const std::string& name, const std::string& body);

	std::string request(
		const std::string& verb, const std::string& name, const std::string& body, 
		std::function<Json::Document(const std::string&, const Json::Document&)>* callback
	);

	Wamp::Message message(const Wamp::Message& message);

	/**
	 * A simple method that provides derived component classes with a 
	 * simple means of overloading this method by calling their own `call()` implementation. 
	 */
	Wamp::Message message(
		const Wamp::Message& message, 
		std::function<Json::Document(const std::string&, const Json::Document&)>* callback
	);

	/**
	 * Notify subscribers to this component of an event
	 * 
	 * @param  event Event string
	 */
	const Component& notify(const Json::Document& event) const;

	Json::Document call(const std::string& name, const Json::Document& args);

	/**
	 * Exception for invalid HTTP request (e.g wrong method name or HTTP method)
	 */
	class RequestInvalidException : public Exception {};

	/**
	 * Exception for invalid Websocket message
	 */
	class MessageInvalidException : public Exception {};

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
	 * A lookup table of Component instances keyed by component address
	 *
	 * Provides a registry of Component instances that
	 * are in memory. Not all component's will be 
	 * in this registry, they have to be registered first using
	 * the `declare()` method
	 */
	static std::map<std::string, Instance> instances_;

	/**
	 * A map of subscribers for each component
	 */
	static std::map<const Component* const, std::vector<int> > subscribers_;

};

/**
 * Generate a HTML document as the basis for a 
 */
template<class Type>
Html::Document Component_page_doc(const Type& component);


template<class Class_>
std::string Component::page_handler(const Component::Instance& instance) {
	return instance.as<const Class_*>()->page();
}

template<class Class_>
std::string Component::request_handler(const Component::Instance& instance, const std::string& verb, const std::string& method, const std::string& body) {
	return instance.as<Class_*>()->request(verb, method, body);
}

template<class Class_>
Wamp::Message Component::message_handler(const Component::Instance& instance, const Wamp::Message& message) {
	return instance.as<Class_*>()->message(message);
}


template<class Class_>
Component* Component::open(Component::Type type, const std::string& path) {
	Class_* component = new Class_;
	component->path(path);
	if (Host::env_var("STENCILA_SESSION").length()) {
		component->restore();
	}
	component->read();
	component->hold(type);
	return component;
}

}
