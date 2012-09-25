/*
Copyright (c) 2012 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//!	@file reflector.hpp
//!	@brief A reflection mechanism for C++

#pragma once

#include <typeinfo>
#include <cxxabi.h>
#include <string>
#include <vector>
#include <map>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Reflect {
    
//Forward declaration required for class Reflection
template<class Derived>
class Reflector;

template <typename Type>
struct HasReflector {
	typedef char (&yes)[1];
	typedef char (&no)[2];
    template <typename A> static yes test(typename A::reflector*);
    template <typename A> static no test(...);
    enum {value = (sizeof(test<Type>(0)) == sizeof(yes))};
};

template <typename Type>
struct IsReflector : std::integral_constant<bool,
    std::is_class<Type>::value and 
    HasReflector<Type>::value
>{};

template<class Derived>
class Reflection {
public:
	
	template<typename Value>
	Derived& data(const char* name, Value* value,const char* desc = 0){
		return static_cast<Derived&>(*this);
	}
	
	template<typename Value>
	Derived& method(const char* name, Value value, const char* desc = 0){
		return static_cast<Derived&>(*this);
	}
    
    template<typename Object>
    Derived& mirror(const std::true_type& is_refector,Object& object){
        Derived& self = static_cast<Derived&>(*this);
        object.reflect(self);
        return self;
    }
    
    template<typename Object>
    Derived& mirror(const std::false_type& is_reflector,Object& object){
        Derived& self = static_cast<Derived&>(*this);
        return self;
    }
    
	template<typename Object>
	Derived& mirror(Object& object) {
		Derived& self = static_cast<Derived&>(*this);
        self.mirror(IsReflector<Object>(),object);
		return self;
	}
    
	template<typename Object>
	Derived& mirror(void) {
		Derived& self = static_cast<Derived&>(*this);
        Object& object = *(static_cast<Object*>(0));
		return self.mirror(object);
	}
};

class Type : public Reflection<Type> {
private:
	std::string type_;

public:
    
	template<typename Object>
	Type& mirror(Object& object) {
		int status;
		type_ = abi::__cxa_demangle(typeid(object).name(), 0, 0, &status);
		return *this;
	}
    
    using Reflection<Type>::mirror;
    
	std::string type(void) const {
		return type_;
	}
};
template<typename Object>
std::string Type_(Object& object = 0){
    Type type;
    type.mirror(object);
    return type.type();
}

class Keys : public Reflection<Keys> {
private:
	std::vector<std::string> keys_;

public:

	template<typename Value>
    Keys& data(const char* name, Value* value, const char* desc = 0){
		keys_.push_back(name);
		return *this;
	}
    
	template<typename Value>
    Keys& method(const char* name, Value value, const char* desc = 0){
		keys_.push_back(name);
		return *this;
	}
	
	std::vector<std::string> keys(void) const {
		return keys_;
	}
};

class Has : public Reflection<Has> {
private:
	std::string name_;
    bool has_;

public:

    Has(const std::string& name):
        name_(name),
        has_(false){
    }

	template<typename Value>
    Has& data(const char* name, Value* value, const char* desc = 0){
		if(not has_) has_ = name==name_;
		return *this;
	}
    
	template<typename Value>
    Has& method(const char* name, Value value, const char* desc = 0){
		if(not has_) has_ = name==name_;
		return *this;
	}
	
	bool has(void) const {
		return has_;
	}
};

class Get : public Reflection<Get> {
private:
	std::string name_;
    void* object_;
    std::string type_;
    
public:

    Get(const std::string& name):
        name_(name),
        object_(0){
    }

	template<typename Value>
    Get& data(const char* name, Value* value, const char* desc = 0){
        if(name==name_){
            object_ = value;
            type_ = Type_(*value);
        }
		return *this;
	}
	
	void* object(void) const {
		return object_;
	}
    
	std::string type(void) const {
		return type_;
	}
};


class Proxy;

template<class Class> class Dispatch;

template<>
class Dispatch<void>{
public:	
	virtual std::string type(void) = 0;
	virtual std::vector<std::string> keys(void) = 0;
    virtual bool has(const std::string& name) = 0;
    virtual Proxy get(void* object, const std::string& name) = 0;
	virtual Proxy create(void) = 0;
};

template<class Class>
class Dispatch : public Dispatch<void>{
public:	

	std::string type(void){
		return Type().mirror<Class>().type();
	};
	
	std::vector<std::string> keys(void){
		return Keys().mirror<Class>().keys();
	};
    
	bool has(const std::string& name){
		return Has(name).mirror<Class>().has();
	};
    
	Proxy get(void* object, const std::string& name);
    
	Proxy create(void);
};

class Registry {
private:
	bool inited_;
	std::map<std::string,Dispatch<void>*>* classes_;


	void init_(void){
		if(not inited_){
			classes_ = new std::map<std::string,Dispatch<void>*>;
			inited_ = true;
		}
	}
public:
	Registry(){
		init_();
	}
	
	~Registry(){
		delete classes_;
	}
	
    template<typename Class>
	void add(Dispatch<void>* dispatch){
		init_();
        const std::string name = Type().mirror<Class>().type();
		classes_->insert(std::pair<std::string,Dispatch<void>*>(name,dispatch));
	}
	
	Dispatch<void>* get(const std::string& name){
		init_();
		auto i = classes_->find(name);
		if(i==classes_->end()) STENCILA_THROW(Exception,"type has not been registered: "+name)
		return i->second;
	}
    
    std::vector<std::string> types(void){
        std::vector<std::string> types;
        for(auto i=classes_->begin();i!=classes_->end();i++) types.push_back(i->first);
        return types;
    }
	
} Registry ;

template<class Type>
struct Register{
	Register(void){
		Registry.add<Type>(new Dispatch<Type>);
	}
};

Register<bool> _bool;
Register<char> _char;
Register<int> _int;
Register<float> _float;
Register<double> _double;

Register<std::string> _std_string;

class Proxy {
private:
	void* object_;
	Dispatch<void>* dispatcher_;
    
public:
	Proxy(void* object, Dispatch<void>* dispatcher):
		object_(object),
		dispatcher_(dispatcher){		
	}
    
    void* pointer(void){
        return object_;
    }
    
	std::string type(void) const {
		return dispatcher_->type();
	}
	
	std::vector<std::string> keys(void) const {
		return dispatcher_->keys();
	}
    
	bool has(const std::string& name){
		return dispatcher_->has(name);
	};
	
	Proxy get(const std::string& name){
		return dispatcher_->get(object_,name);
	}
    
	Proxy operator[](const std::string& name){
		return get(name);
	}
};

template<class Class>
Proxy Dispatch<Class>::get(void* object, const std::string& name){
    Get get(name);
    get.mirror(*static_cast<Class*>(object));
    return Proxy(get.object(),Registry.get(get.type()));
};

template<class Class>
Proxy Dispatch<Class>::create(void){
    return Proxy(new Class,this);
};

Proxy Create(const std::string& name){
	return Registry.get(name)->create();
}
    
template<class Derived>
class Reflector {
public:

    typedef Reflector<Derived> reflector;

	template<typename... Args>
	static Derived& create(Args... args) {
		//Use a shared pointer for carbage collection?
		Derived* o = new Derived(args...);
		return *o;
	}

	template<class Reflection>
	void reflect(Reflection& r){
		static_cast<Derived*>(this)->reflect(r);
	}
    
	std::string type(void) {
		Type type;
		type.mirror(*static_cast<Derived*>(this));
		return type.type();
	}
    
	std::vector<std::string> keys(void) {
		Keys keys;
		keys.mirror(*static_cast<Derived*>(this));
		return keys.keys();
	}
    
	bool has(const std::string& name) {
		Has has(name);
		has.mirror(*static_cast<Derived*>(this));
		return has.has();
	}
    
	Proxy get(const std::string& name) {
		Get get(name);
		get.mirror(*static_cast<Derived*>(this));
		return Proxy(get.object(),Registry.get(get.type()));
	}
};

/*
Note the use of variadic macros (the ellipses and the "##__VA_ARGS__" have special meaning).
See http://gcc.gnu.org/onlinedocs/cpp/Variadic-Macros.html
*/

#define REFLECT(_ATTRS) \
template<class Reflection> \
void reflect(Reflection& r){\
    _ATTRS \
}

#define TYPE(_NAME,...) r.type(#_NAME,##__VA_ARGS__);

#define DATA(_NAME,...) r.data(#_NAME,&_NAME,##__VA_ARGS__);

#define METHOD(_NAME,...) {\
    typedef std::remove_reference<decltype(*this)>::type type;\
    r.method(#_NAME,&type::_NAME,##__VA_ARGS__);\
}

#define REGISTER(_NAME) { Register<_NAME> _(#_NAME); }

}
}