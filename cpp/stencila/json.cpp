#include <fstream>

#include <json/json.h>
#include <jsoncpp.cpp>

#include <stencila/json.hpp>

namespace Stencila {
namespace Json {

template<>
bool Node::is<void>(void) const {
	return pimpl_->isNull();
}

template<>
bool Node::is<bool>(void) const {
	return pimpl_->isBool();
}

template<>
bool Node::is<int>(void) const {
	return pimpl_->isInt();
}

template<>
bool Node::is<unsigned int>(void) const {
	return pimpl_->isUInt();
}

template<>
bool Node::is<float>(void) const {
	return pimpl_->isNumeric();
}

template<>
bool Node::is<double>(void) const {
	return pimpl_->isDouble();
}

template<>
bool Node::is<std::string>(void) const {
	return pimpl_->isString();
}

template<>
bool Node::is<Array>(void) const {
	return pimpl_->isArray();
}

template<>
bool Node::is<Object>(void) const {
	return pimpl_->isObject();
}

template<>
bool Node::as<bool>(void) const {
	return pimpl_->asBool();
}

template<>
int Node::as<int>(void) const {
	return pimpl_->asInt();
}

template<>
unsigned int Node::as<unsigned int>(void) const {
	return pimpl_->asUInt();
}

template<>
float Node::as<float>(void) const {
	return pimpl_->asFloat();
}

template<>
double Node::as<double>(void) const {
	return pimpl_->asDouble();
}

template<>
std::string Node::as<std::string>(void) const {
	return pimpl_->asString();
}

template<>
std::vector<std::string> Node::as<std::vector<std::string>>(void) const {
	auto size = pimpl_->size();
	std::vector<std::string> result(size);
	int index = 0;
	for(auto value : *pimpl_) result[index++] = value.asString();
	return result;
}

template<>
std::map<std::string,std::string> Node::as<std::map<std::string,std::string>>(void) const {
	std::map<std::string,std::string> result;
	auto names = pimpl_->getMemberNames();
	int index = 0;
	for(auto value : *pimpl_) result[names[index++]] = value.asString();
	return result;
}

unsigned int Node::size(void) const{
	if(is<Object>() or is<Array>()) return pimpl_->size();
	else return 0u;
}

bool Node::has(const std::string& name) const {
	if(is<Object>()) return pimpl_->isMember(name);
	else return false;
}

Node Node::operator[](const std::string& name){
	return (*pimpl_)[name];
}

const Node Node::operator[](const std::string& name) const {
	return (*pimpl_)[name];
}

Node Node::operator[](const unsigned int& index){
	return (*pimpl_)[index];
}

const Node Node::operator[](const unsigned int& index) const {
	return (*pimpl_)[index];
}

#define APPEND_VALUE(TYPE_) \
	template<> \
	Node Node::append(TYPE_ value){ \
		pimpl_->append(value); \
		return (*pimpl_)[pimpl_->size()-1];	\
	} \
	template<> \
	Node Node::append(const std::string& name,TYPE_ value){ \
		(*pimpl_)[name] = value; \
		return (*pimpl_)[name];	\
	}

APPEND_VALUE(bool)
APPEND_VALUE(int)
APPEND_VALUE(unsigned int)
APPEND_VALUE(float)
APPEND_VALUE(double)
APPEND_VALUE(const char*)
APPEND_VALUE(std::string)

#undef APPEND_VALUE

template<>
Node Node::append(Object){
	JsonCpp::Value object(JsonCpp::objectValue);
	pimpl_->append(object);
	return object;
}

template<>
Node Node::append(const std::string& name,Object){
	JsonCpp::Value object(JsonCpp::objectValue);
	(*pimpl_)[name] = object;
	return object;	
}

template<>
Node Node::append(Array){
	JsonCpp::Value array(JsonCpp::arrayValue);
	pimpl_->append(array);
	return array;
}

template<>
Node Node::append(const std::string& name,Array){
	JsonCpp::Value array(JsonCpp::arrayValue);
	(*pimpl_)[name] = array;
	return array;	
}

#define APPEND_VECTOR(TYPE_) \
	template<> \
	Node Node::append(const std::vector<TYPE_>& values){ \
		JsonCpp::Value array(JsonCpp::arrayValue); \
		for(auto value : values) array.append(value); \
		pimpl_->append(array); \
		return array; \
	} \
	template<> \
	Node Node::append(const std::string& name, const std::vector<TYPE_>& values){ \
		JsonCpp::Value array(JsonCpp::arrayValue); \
		for(auto value : values) array.append(value); \
		(*pimpl_)[name] = array; \
		return array; \
	}

APPEND_VECTOR(bool)
APPEND_VECTOR(int)
APPEND_VECTOR(unsigned int)
APPEND_VECTOR(float)
APPEND_VECTOR(double)
APPEND_VECTOR(const char*)
APPEND_VECTOR(std::string)

#undef APPEND_VECTOR

#define APPEND_MAP(TYPE_) \
	template<> \
	Node Node::append(const std::map<std::string,TYPE_>& values){ \
		JsonCpp::Value object(JsonCpp::objectValue); \
		for(auto value : values) object[value.first] = value.second; \
		pimpl_->append(object); \
		return object; \
	} \
	template<> \
	Node Node::append(const std::string& name,const std::map<std::string,TYPE_>& values){ \
		JsonCpp::Value object(JsonCpp::objectValue); \
		for(auto value : values) object[value.first] = value.second; \
		(*pimpl_)[name] = object; \
		return object; \
	}

APPEND_MAP(bool)
APPEND_MAP(int)
APPEND_MAP(unsigned int)
APPEND_MAP(float)
APPEND_MAP(double)
APPEND_MAP(const char*)
APPEND_MAP(std::string)

#undef APPEND_MAP

Node& Node::load(const std::string& json){
	JsonCpp::Reader reader;
	pimpl_->clear();
	bool ok = reader.parse(json,*pimpl_);
	if(not ok){
		STENCILA_THROW(Exception,reader.getFormattedErrorMessages());
	}
	return *this;
}

std::string Node::dump(bool pretty) const {
	if(pretty){
		JsonCpp::StyledWriter writer;
		return writer.write(*pimpl_);	
	} else {
		JsonCpp::FastWriter writer;
		writer.omitEndingLineFeed();
		return writer.write(*pimpl_);		
	}
}

Document::Document(void):
	Node(new Impl){
}

Document::Document(const Document& other):
	Node(new Impl(*other.pimpl_)){
}

Document::Document(const Object& object):
	Node(new Impl(JsonCpp::objectValue)){
}

Document::Document(const Array& array):
	Node(new Impl(JsonCpp::arrayValue)){
}

Document::Document(const char* json):
	Node(new Impl){
	load(json);
}

Document::Document(const std::string& json):
	Node(new Impl){
	load(json);
}

Document::~Document(void){
	delete pimpl_;
}

Document& Document::read(std::istream& stream){
	std::stringstream string;
	string<<stream.rdbuf();
	load(string.str());
	return *this;
}

Document& Document::read(const std::string& path){
	std::ifstream file(path);
	return read(file);
}

const Document& Document::write(std::ostream& stream) const {
	stream<<dump();
	return *this;
}

const Document& Document::write(const std::string& path) const {
	std::ofstream file(path);
	return write(file);
}


}
}
