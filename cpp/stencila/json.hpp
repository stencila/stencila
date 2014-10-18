#pragma once

#include <string>
#include <vector>
#include <map>

#include <stencila/exception.hpp>

namespace Json {
	class Value;
}
namespace JsonCpp = Json;

namespace Stencila {
namespace Json {

/**
 * @namespace  Stencila::Json
 *
 * This namespace contains utility classes for 
 * handling the [JavaScript Object Notation (JSON)](http://www.json.org/).
 */

/**
* @struct Object
* 
* This struct simply allows for syntax such as 
* @code is<Object>() @endcode
*/
struct Object {};

/**
* @struct Array
* 
* This struct simply allows for syntax such as 
* @code is<Array>(document["list"]) @endcode
*/
struct Array {};

/**
 * A JSON Node
 */
class Node {
public:

	/**
	 * Is this Node of this type?
	 */
	template<typename Type>
	bool is(void) const;

	/**
	 * Convert this Node to this type
	 */
	template<typename Type>
	Type as(void) const;

	/**
	 * Get the number of child nodes in this Node
	 */
	unsigned int size(void) const;

	/**
	 * Does the object have a child node with the given name
	 *
	 * @param name Name being searched for
	 */
	bool has(const std::string& name) const;

	Node operator[](const std::string& name);

	Node operator[](const unsigned int& index);

	template<class Type>
	Node append(Type value);

	template<class Type>
	Node append(const std::string& name, Type value);

	template<class Type>
	Node append(const std::vector<Type>& values);

	template<class Type>
	Node append(const std::string& name, const std::vector<Type>& values);

	template<class Type>
	Node append(const std::map<std::string,Type>& values);

	template<class Type>
	Node append(const std::string& name, const std::map<std::string,Type>& values);

protected:
	typedef ::Json::Value Impl;

	Node(Impl& impl):
		pimpl_(&impl){};

	Node(Impl* impl):
		pimpl_(impl){};

	Impl* pimpl_;
};

/**
 * A JSON Document
 */
class Document : public Node {
public:

	Document(void);

	Document(const Document& other);

	Document(const Object& object);

	Document(const Array& array);

	Document(const char* json);

	Document(const std::string& json);

	~Document(void);

	/**
	* Load a JSON string into this document
	*
	* @param json A std::string of JSON
	*/
	Document& load(const std::string& json);

	/**
	* Dump this document to a string
	*
	* @param pretty Prettify the output?
	*/
	std::string dump(bool pretty = false) const;

	Document& read(std::istream& stream);

	Document& read(const std::string& path);

	const Document& write(std::ostream& stream) const;

	const Document& write(const std::string& path) const;
};

}
}
