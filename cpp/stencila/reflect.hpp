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

//! @file reflector.hpp
//! @brief A reflection mechanism for C++
//! @author Nokome Bentley

#pragma once

#include <typeinfo>
#include <cxxabi.h>
#include <string>
#include <vector>
#include <map>

#include <stencila/exception.hpp>
#include <stencila/print.hpp>

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

    //! @brief 
    //! @param name
    //! @param value
    //! @param desc
    //! @return 
    template<typename Value>
    Derived& data(const char* name, Value* value,const char* desc = 0){
        return static_cast<Derived&>(*this);
    }

    //! @brief 
    //! @param name
    //! @param value
    //! @param desc
    //! @return 
    template<typename Value>
    Derived& method(const char* name, Value value, const char* desc = 0){
      return static_cast<Derived&>(*this);
    }
    
    //! @brief 
    //! @param refector
    //! @param object   
    //! @return 
    template<typename Object>
    Derived& mirror(const std::true_type& is_refector,Object& object){
        Derived& self = static_cast<Derived&>(*this);
        object.reflect(self);
        return self;
    }
    
    //! @brief 
    //! @param reflector
    //! @param object
    //! @return 
    template<typename Object>
    Derived& mirror(const std::false_type& is_reflector,Object& object){
        Derived& self = static_cast<Derived&>(*this);
        return self;
    }
    
    //! @brief 
    //! @param object
    //! @return 
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

    //! @brief 
    //! @param object
    //! @param name
    //! @param status
    //! @return 
    template<typename Object>
    Type& mirror(Object& object) {
        int status;
        type_ = abi::__cxa_demangle(typeid(object).name(), 0, 0, &status);
        return *this;
    }

    using Reflection<Type>::mirror;

    //! @brief 
    //! @return 
    std::string type(void) const {
        return type_;
    }

    //! @brief 
    //! @param object
    //! @return 
    template<typename Object>
    static std::string type(Object& object){
        Type type;
        type.mirror(object);
        return type.type();
    }   
};

class Repr : public Reflection<Repr> {
private:
    std::string repr_;

public:

    //! @brief 
    Repr(void):
        repr_(""){
    }
    
    //! @brief 
    //! @param name 
    //! @param value
    //! @param desc
    //! @return 
    template<typename Value>
    Repr& data(const char* name, Value* value, const char* desc = 0){
        repr_ += std::string(name) + ":";
        mirror(*value);
        repr_ += ",";
        return *this;
    }
        
    using Reflection<Repr>::mirror;
    
    //! @brief 
    //! @param reflector
    //! @param object
    //! @return 
    template<typename Object>
    Repr& mirror(const std::true_type& is_reflector,Object& object){
        repr_ += Type::type(object) + "{";
        object.reflect(*this);
        repr_ += "}";
        return *this;
    }
    
    //! @brief 
    //! @param reflector
    //! @param object
    //! @return 
    template<typename Object>
    Repr& mirror(const std::false_type& is_reflector,Object& object){
        std::string printed = print()<<object;
        repr_ += printed;
        return *this;
    }
    //! @brief 
    //! @return 
    std::string repr(void) const {
        return repr_;
    }
    
    //! @brief 
    //! @param object
    //! @return 
    template<typename Object>
    static std::string repr(Object& object){
        Repr repr;
        repr.mirror(object);
        return repr.repr();
    }
};

class Keys : public Reflection<Keys> {
private:
    std::vector<std::string> keys_;

public:

    //! @brief 
    //! @param name
    //! @param value
    //! @param desc
    //! @return 
    template<typename Value>
    Keys& data(const char* name, Value* value, const char* desc = 0){
        keys_.push_back(name);
        return *this;
    }
    
    //! @brief 
    //! @param name
    //! @param value
    //! @param desc
    //! @return 
    template<typename Value>
    Keys& method(const char* name, Value value, const char* desc = 0){
        keys_.push_back(name);
        return *this;
    }
  
    //! @brief 
    //! @return 
    std::vector<std::string> keys(void) const {
        return keys_;
    }
    
    //! @brief 
    //! @param object
    //! @return 
    template<typename Object>
    static std::vector<std::string> keys(Object& object){
        Keys keys;
        keys.mirror(object);
        return keys.keys();
    }
};

class Has : public Reflection<Has> {
private:
    std::string name_;
    bool has_;

public:

    //! @brief 
    //! @param name
    //! @return 
    Has(const std::string& name):
        name_(name),
        has_(false){
    }

    //! @brief 
    //! @param name
    //! @param value    
    //! @param desc
    //! @return 
    template<typename Value>
    Has& data(const char* name, Value* value, const char* desc = 0){
        if(not has_) has_ = name==name_;
        return *this;
    }
     
     //! @brief 
     //! @param name
     //! @param value
     //! @param desc
     //! @return 
    template<typename Value>
    Has& method(const char* name, Value value, const char* desc = 0){
        if(not has_) has_ = name==name_;
        return *this;
    }

    //! @brief 
    //! @return 
    bool has(void) const {
        return has_;
    }
    
    //! @brief 
    //! @param object
    //! @param name
    //! @return 
    template<typename Object>
    static bool has(Object& object,const std::string& name){
        Has has(name);
        has.mirror(object);
        return has.has();
    }
};

class Get : public Reflection<Get> {
private:
    std::string name_;
    void* object_;
    std::string type_;
    
public:

    //! @brief 
    //! @param name
    //! @return 
    Get(const std::string& name):
        name_(name),
        object_(0),
        type_(""){
    }

    //! @brief 
    //! @param name
    //! @param value
    //! @param desc
    //! @return 
    template<typename Value>
    Get& data(const char* name, Value* value, const char* desc = 0){
        if(name==name_){
            object_ = value;
            type_ = Type::type(*value);
        }
        return *this;
    }
    
    //! @brief 
    void exception(void) const {
        STENCILA_THROW(Exception,"object does not have key:"+name_);
    }
    
    //! @brief 
    void* object(void) const {
        if(object_) return object_;
        else exception();
        return 0;
    }
    
    //! @brief 
    //! @return 
    std::string type(void) const {
        if(object_) return type_;
        else exception();
        return "";
    }
};


class Proxy;

template<class Class> class Dispatch;

template<>
class Dispatch<void>{
public:
    //! @brief 
    //! @return 
    virtual Proxy create(void);
    
    //! @brief 
    //! @return 
    virtual std::string type(void* object){
        return "void";
    };
    
    //! @brief 
    //! @return 
    virtual std::vector<std::string> keys(void* object){
        return {};
    };
    
    //! @brief 
    //! @param name 
    //! @return 
    virtual bool has(void* object, const std::string& name){
            return false;
    };
    
    //! @brief 
    //! @param object
    //! @param name
    //! @return 
    virtual Proxy get(void* object, const std::string& name);
    
    virtual std::string repr(void* object){
        return "";
    };
};

template<class Class>
class Dispatch : public Dispatch<void>{
public:                                           

    Proxy create(void);

    //! @brief 
    //! @return 
    std::string type(void* object){
        return Type::type(*static_cast<Class*>(object));
    };

    //! @brief 
    //! @return 
    std::vector<std::string> keys(void* object){
        return Keys::keys(*static_cast<Class*>(object));
    };

    //! @brief 
    //! @param name
    //! @return 
    bool has(void* object, const std::string& name){
        return Has::has(*static_cast<Class*>(object),name);
    };

    //! @brief 
    //! @param object   
    //! @param name
    //! @return 
    Proxy get(void* object, const std::string& name);

    //! @brief
    //! @return  
    std::string repr(void* object){
        return Repr::repr(*static_cast<Class*>(object));
    }
};

class Registry {
private:
    bool inited_;
    std::map<std::string,Dispatch<void>*>* classes_;
    static Dispatch<void> dispatch_void_;

    //! @brief 
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
    
    //! @brief 
    //! @param dispatch
    template<typename Class>
    void add(Dispatch<void>* dispatch){
        init_();
        const std::string name = Type().mirror<Class>().type();
        classes_->insert(std::pair<std::string,Dispatch<void>*>(name,dispatch));
    }
    
    //! @brief 
    //! @param name
    //! @return 
    Dispatch<void>* get(const std::string& name){
    init_();
        auto i = classes_->find(name);
        if(i==classes_->end()) return &dispatch_void_;
        else return i->second;
    }
    
    //! @brief 
    //! @return 
    std::vector<std::string> types(void){
        std::vector<std::string> types;
        for(auto i=classes_->begin();i!=classes_->end();i++) types.push_back(i->first);
        return types;
    }

} Registry ;

Dispatch<void> Registry::dispatch_void_;

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
    //! @brief 
    //! @param dispatcher
    //! @return 
    Proxy(void* object, Dispatch<void>* dispatcher):
        object_(object),
        dispatcher_(dispatcher){
    }
    //! @brief 
    void* object(void){
        return object_;
    }

    //! @brief 
    //! @return 
    std::string type(void) const {
        return dispatcher_->type(object_);
    }
    
    //! @brief  
    std::vector<std::string> keys(void) const {
        return dispatcher_->keys(object_);
    }

    //! @brief 
    //! @param name
    //! @return 
    bool has(const std::string& name){
        return dispatcher_->has(object_,name);
    };

    //! @brief 
    //! @param name
    //! @return 
    Proxy get(const std::string& name){
        return dispatcher_->get(object_,name);
    }

//! @brief 
//! @param name
//! @return 
    Proxy operator[](const std::string& name){
        return get(name);
    }
    //! @brief 
    //! @return 
    std::string repr(void) const {
        return dispatcher_->repr(object_);
    }
};

//! @{
//! Definitions for Dispatch<void> methods involving Proxy
inline Proxy Dispatch<void>::create(void){
    STENCILA_THROW(Exception,"unable to create Proxy for Dispatch<void>");
};

//! @brief 
//! @param object
//! @param name
//! @return 
inline Proxy Dispatch<void>::get(void* object, const std::string& name){
    STENCILA_THROW(Exception,"no keys");
};
//! @}

//! @{
//! Definitions for Dispatch<Class> methods involving Proxy
template<class Class>
Proxy Dispatch<Class>::create(void){
    return Proxy(new Class,this);
};

//! @brief 
//! @param object
//! @param name
//! @return 
template<class Class>
Proxy Dispatch<Class>::get(void* object, const std::string& name){
    Get get(name);
    get.mirror(*static_cast<Class*>(object));
    return Proxy(get.object(),Registry.get(get.type()));
};
//! @}

// Convienience function for creating objects
Proxy Create(const std::string& name){
    return Registry.get(name)->create();
}

template<class Derived>
class Reflector {
public:

    typedef Reflector<Derived> reflector;

    template<typename... Args>
    static Derived& create(Args... args) {
          //Use a shared pointer for garbage collection?
          Derived* o = new Derived(args...);
          return *o;
    }

    template<class Reflection>
    void reflect(Reflection& r){
          static_cast<Derived*>(this)->reflect(r);
    }

    Proxy proxy(void) {
          return Proxy(this,Registry.get(type()));
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

    std::string repr(void) const{
          Repr repr();
          repr.mirror(*static_cast<Derived*>(this));
          return repr.repr();
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

#define DATA(_NAME,...) r.data(#_NAME,&_NAME,##__VA_ARGS__);

#define METHOD(_NAME,...) {\
    typedef std::remove_reference<decltype(*this)>::type type;\
    r.method(#_NAME,&type::_NAME,##__VA_ARGS__);\
}

#define REGISTER(_NAME) { Register<_NAME> _(#_NAME); }

}
}
