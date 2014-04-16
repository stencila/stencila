#pragma once

#include <string>
#include <vector>

#include <boost/algorithm/string.hpp>

#include <rapidjson/rapidjson.h>
#include <rapidjson/document.h>
#include <rapidjson/prettywriter.h>
#include <rapidjson/filestream.h>
#include <rapidjson/stringbuffer.h>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Json {

/**
 @namespace  Json

 This namespace contains utility classes for handling the <a href="http://www.json.org/">JavaScript Object Notation (JSON)</a>.

 The Stencila library currently uses <a href="https://code.google.com/p/rapidjson">rapidjson</a> as a backend for JSON parsing and generation.
 We chose rapidjson because it has <a href="https://code.google.com/p/rapidjson/wiki/Performance">good performance</a>
 and a pretty usable API. Documentation for rapidjson are available at cpp/requirements/rapidjson-0.1/doc/html/index.html
 and there is an example of usage <a href="https://code.google.com/p/rapidjson/source/browse/trunk/example/tutorial/tutorial.cpp">here</a>

 There are a large number of other C/C++JSON libraries including:
    - <a href="http://jsoncpp.sourceforge.net/">JsonCpp</a>
    - <a href="http://www.codeproject.com/KB/recipes/JSON_Spirit.aspx">JSON Spirit</a>
    - <a href="http://lloyd.github.com/yajl/">YAJL</a>
 which, if there is good reason to do so, we might switch to one day.
*/

/*! 
* @typedef Value
* 
* As part of its strategy for producing a fast JSON parser, rapidjson prevents copying of rapidjson::Value objects by 
* making the copy constructor private. This causes lots of problems with trying to derive from that class. 
* For that reason Stencila just uses that class as is.
*/
typedef rapidjson::Value Value;

/**
* @class Object
* 
* This class simply allows for syntax such as 
* @code is<Object>() @endcode
*/

class Object {};

/**
* @class Array
* 
* This class simply allows for syntax such as 
* @code is<Array>(document["list"]) @endcode
*/
class Array {};

/**
 * Is a `Value` of a given type?
 *
 * @param value JSON `Value`
 */
template<typename Type> 
bool is(const Value& value);

#define STENCILA_LOCAL(type,method) template<> inline bool is<type>(const Value& value) { return value.method(); }
	STENCILA_LOCAL(void,IsNull)
	STENCILA_LOCAL(bool,IsBool)
	STENCILA_LOCAL(int,IsInt)
	STENCILA_LOCAL(double,IsDouble)
	STENCILA_LOCAL(std::string,IsString)
	STENCILA_LOCAL(Stencila::Json::Object,IsObject)
	STENCILA_LOCAL(Stencila::Json::Array,IsArray)
#undef STENCILA_LOCAL

/**
 * Convert a `Value` to another type
 *
 * @param value JSON `Value`
 */
template<typename Type> 
Type as(const Value& value);

#define STENCILA_LOCAL(type,method) template<> inline type as<type>(const Value& value) { return value.method(); }
	STENCILA_LOCAL(bool,GetBool)
	STENCILA_LOCAL(int,GetInt)
	STENCILA_LOCAL(double,GetDouble)
	STENCILA_LOCAL(std::string,GetString)
#undef STENCILA_LOCAL

/**
 * Does the object have a member with the given name
 *
 * @param value JSON `Object`
 * @param name Name being searched for
 */
static bool has(const Value& value,const std::string& name) {
	return is<Object>(value)?value.HasMember(name.c_str()):false;
}

/**
 * Size of a JSON array
 * 
 * @param  value JSON `Array`
 */
static uint size(const Value& value) {
	return is<Array>(value)?value.Size():0;
}

/**
 * A JSON Document
 */
class Document : public rapidjson::Document {
public:
	
	Document(void){
        // Make this document an object
        SetObject();
	}

    Document(const char* json){
        load(json);
    }

	Document(const std::string& json){
		load(json);
	}

    Document(const Document& other){
        DeepCopy(*this,other,true);
    }

    /**
     * Load a JSON string into the `Document`
     *
     * @param json A std::string of JSON
     */
    Document& load(const std::string& json){
        std::string input = json;
        boost::algorithm::trim(input);
        if(input.length()==0){
            // Make this document an object if there is no JSON
            SetObject();
        } else {
            // Otherwise parse the JSON
            Parse<0>(input.c_str());
            if(HasParseError()) {
                STENCILA_THROW(Exception,std::string("JSON parsing error: ")+GetParseError()+": "+json);
            }
        }
        return *this;
    }

    /**
     * Dump document to a string
     *
     * @return JSON string of document
     */
    std::string dump(void) {
          rapidjson::StringBuffer buffer;
          rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);
          Accept(writer);
          return buffer.GetString();
    }

    /**
     * Pretty print document to a string
     *
     * @return JSON string of document with indentation
     */
    std::string pretty(void) {
          rapidjson::StringBuffer buffer;
          rapidjson::PrettyWriter<rapidjson::StringBuffer> writer(buffer);
          Accept(writer);
          return buffer.GetString();
    }

    /**
     * Get a member of the document
     *
     * Override of base method to allow for string arguments
     */
    Value& operator[](const std::string& name) {
        return rapidjson::Document::operator[](name.c_str());
    }
    const Value& operator[](const std::string& name) const {
        return rapidjson::Document::operator[](name.c_str());
    }

    /**
     * Append a member to a JSON `Value`
     */
    Document& append(Value& to,Value& name,Value& value) {
    	to.AddMember(name,value,GetAllocator());
        return *this;
    }

    Document& append(Value& to,const std::string& name,Value& value) {
        Value name_value(name.c_str(),name.length(),GetAllocator());
        return append(to,name_value,value);
    }

    Document& append(Value& to,const std::string& name,const int& value) {
        Value int_value(value);
        return append(to,name,int_value);
    }

    Document& append(Value& to,const std::string& name,const std::string& value) {
        Value string_value(value.c_str(),value.length(),GetAllocator());
        return append(to,name,string_value);
    }

    template<typename Type>
    Document& append(Value& to,const std::string& name,const std::vector<Type>& items) {
        Value array_value(rapidjson::kArrayType);
        for(auto item : items) array_value.PushBack(item,GetAllocator());
        return append(to,name,array_value);
    }

    template<typename Type>
    Document& append(const std::string& name, const Type& value) {
          return append(*this, name, value);
    }
    
    Document& append(const std::string& name, Value& value) {
          return append(*this, name, value);
    }
};

}
}
