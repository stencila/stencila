/*
Copyright (c) 2012-2013 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file json.hpp
//! @brief Classes and functions for working with JSON
//! @author Nokome Bentley

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

/*! 
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
 @typedef Value
 
 As part of its strategy for producing a fast JSON parser, rapidjson prevents copying of rapidjson::Value objects by 
 making the copy constructor private. This causes lots of problems with trying to derive from that class. 
 For that reason Stencila just uses that class as is.
*/
typedef rapidjson::Value Value;

/*! 
 @class Object
 
 This class simply allows for syntax such as 
 @code document.is<Object>() @endcode
 @see Document
*/

class Object{};
const rapidjson::Type ObjectType = rapidjson::kObjectType;

/*! 
 @class Array
 
 This class simply allows for syntax such as 
 @code document.is<Array>(document["list"]) @endcode
 @see Document
*/
class Array{};
const rapidjson::Type ArrayType = rapidjson::kArrayType;

/*! 
 @class Document
 @brief A JSON document
*/
class Document {
private:

    rapidjson::Document doc_;
    
public:

    Document(const std::string& json=""){
        load(json);
    }

public:
    //! @brief Load a JSON string into the Document
    //! @param json a std::string of JSON
    //! @return the Document
    Document& load(const std::string& json){
        std::string input = json;
        boost::algorithm::trim(input);
        if(input.length()==0) input = "{}";
        doc_.Parse<0>(input.c_str());
        if(doc_.HasParseError()) {
            STENCILA_THROW(Exception,std::string("JSON parsing error: ")+doc_.GetParseError()+": "+json);
        }
        return *this;
    }

    //! @brief Is a value an instance of Type?
    //! @param value The value to be tested
    template<typename Type>
    bool is(const Value& value) const;

    //! @brief Is the document an instance of Type?
    template<typename Type>
    bool is(void) const;

    //! @brief Return an object of Type from the value
    template<typename Type>
    Type as(const Value& value) const ;

    //! @brief Return an object of Type from the document
    template<typename Type>
    Type as(void) const;

    //! @brief Does the value have a member called ...?
    //! @param value Value being searched
    //! @param name Name being searched for
    bool has(const Value& value,const std::string& name) const {
          return value.HasMember(name.c_str());
    }

    //! @brief Does the document have a member called ...?
    //! @param name Name being searched for
    bool has(const std::string& name) const {
          return has(doc_,name);
    }
    
    Value& get(const std::string& name) {
        return doc_[name.c_str()];
    }
    const Value& get(const std::string& name) const {
        return doc_[name.c_str()];
    }
    
    Value& operator[](const std::string& name) {
        return get(name);
    }
    const Value& operator[](const std::string& name) const {
        return get(name);
    }

    //! @brief Add a member to a value
    //! @param to Value to which the member will be added
    //! @param name Name of the member
    //! @param value Value of the member
    Document& add(Value& to,const std::string& name,const int& value) {
        rapidjson::Document::AllocatorType& allocator = doc_.GetAllocator();
        Value name_value(name.c_str(),name.length(),allocator);
        Value item_value(value);
        to.AddMember(name_value,item_value,allocator);
        return *this;
    }

    Document& add(Value& to,const std::string& name,const std::string& value) {
        rapidjson::Document::AllocatorType& allocator = doc_.GetAllocator();
        Value name_value(name.c_str(),name.length(),allocator);
        Value item_value(value.c_str(),value.length(),allocator);
        to.AddMember(name_value,item_value,allocator);
        return *this;
    }
    
    Document& add(Value& to,const std::string& name,Value& value) {
        rapidjson::Document::AllocatorType& allocator = doc_.GetAllocator();
        Value name_value(name.c_str(),name.length(),allocator);
        to.AddMember(name_value,value,allocator);
        return *this;
    }

    //! @brief Add a member to the document
    //! @param name Name of the member
    //! @param value Value of the member
    template<typename Type>
    Document& add(const std::string& name, const Type& value) {
          return add(doc_, name, value);
    }
    
    Document& add(const std::string& name, Value& value) {
          return add(doc_, name, value);
    }
    
    //! @brief Append an item to an array
    //! @param to Array to which the item will be appended
    //! @param value Value of the member
    Document& append(Value& to, Value& value) {
        rapidjson::Document::AllocatorType& allocator = doc_.GetAllocator();
        to.PushBack(value,allocator);
        return *this;
    }

    //! @brief Print document to a string
    //! @return JSON string of document
    std::string dump(void) {
          rapidjson::StringBuffer buffer;
          rapidjson::Writer<rapidjson::StringBuffer> writer(buffer);
          doc_.Accept(writer);
          return buffer.GetString();
    }

    //! @brief Pretty print document to a string
    //! @return JSON string of document with indentation
    std::string pretty(void) {
          rapidjson::StringBuffer buffer;
          rapidjson::PrettyWriter<rapidjson::StringBuffer> writer(buffer);
          doc_.Accept(writer);
          return buffer.GetString();
    }
};

#define IS(type,method)\
template<> inline \
bool Document::is<type>(const Value& value) const{ \
    return value.method(); \
} \
template<> inline \
bool Document::is<type>(void) const{ \
    return is<type>(doc_); \
}

IS(void,IsNull)
IS(bool,IsBool)
IS(int,IsInt)
IS(double,IsDouble)
IS(std::string,IsString)
IS(Stencila::Json::Object,IsObject)
IS(Stencila::Json::Array,IsArray)

#undef IS

#define AS(type,method)\
template<> inline \
type Document::as<type>(const Value& value) const{ \
    return value.method(); \
} \
template<> inline \
type Document::as<type>(void) const{ \
    return as<type>(doc_); \
} \

AS(bool,GetBool)
AS(int,GetInt)
AS(double,GetDouble)
AS(std::string,GetString)

#undef AS

template<>
inline
std::vector<int> Document::as<std::vector<int>>(const Value& value) const {
    std::vector<int> vec;
    for(auto i = value.Begin();i != value.End();i++){
        vec.push_back(i->GetInt());
    }
    return vec;
}

}
}
