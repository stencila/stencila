/*
Copyright (c) 2013 Stencila Ltd

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is 
hereby granted, provided that the above copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD 
TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. 
IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR 
CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA
OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, 
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

//! @file component.cpp
//! @brief Implementation of a component static members
//! @author Nokome Bentley

#include <stencila/component.hpp>

#include <stencila/format.hpp>
#include <stencila/stencil.hpp>
#include <stencila/theme.hpp>

namespace Stencila {

boost::uuids::random_generator Id::generator_;
    
std::map<Id,Component<>::Registration> Component<>::registry_;

template<class Class>
Class& Component<>::create(void){
    return *(new Class);
}

template<class Class>
Class& Component<>::create(const Id& id){
    return *(new Class(id));
}

template<class Class>
Class* Component<>::obtain(const Id& id){
    std::string type = Class::type();
    auto i = registry_.find(id);
    if(i!=registry_.end()){
        if(i->second.type==type) return static_cast<Class*>(i->second.pointer);
        else return 0;
    } else {
        std::string dir = directory(id);
        if(boost::filesystem::exists(dir)){
            Class& component = create<Class>(id);
            return &component;
        } else {
            return 0;
        }
    }
}

template<class Class>
std::vector<Class*> Component<>::filter(void){
    std::string type = Class::type();
    std::vector<Class*> filtered;
    for(const std::map<Id,Registration>::value_type& i : registry_){
        if(i.second.type==type) filtered.push_back(static_cast<Class*>(i.second.pointer));
    }
    return filtered;
}

std::string Component<>::rest(const std::string& method, const std::string& uri, const std::string& json){
    return rest(
        Http::Method(method),
        Http::Uri(uri),
        json
    );
}

std::string Component<>::rest(const Http::Method& verb, const Http::Uri& uri, const std::string& json){
    try{
        std::string type = uri.path(0);
        if(type.length()==0) return R"({"error":"type not specified"})";
        if(type=="stencil") return rest_type<Stencil>(verb,uri,json);
        if(type=="theme") return rest_type<Theme>(verb,uri,json);
        return Format(R"({"error":"unsupported type: %s"})")<<type;
    } catch (std::exception &e) {
        return Format(R"({"error":"%s"})")<<e.what();
    } catch (...) {
        return R"({error:unknown})";
    }
}

template<class Class>
std::string Component<>::rest_type(const Http::Method& verb, const Http::Uri& uri, const std::string& json){
    if(verb==Http::Post) return post<Class>(uri,json);
    else if(verb==Http::Get) return get<Class>(uri);
    else if(verb==Http::Put) return put<Class>(uri,json);
    else if(verb==Http::Delete) return del<Class>(uri);
    else return Format(R"({"error":"unsupported method: %s"})")<<verb;
}

template<class Class>
std::string Component<>::post(const Http::Uri& uri, const std::string& in){
    Class& component = create<Class>();
    component.put(in);
    return Format(R"({"id":"%s"})")<<component.id();
}

template<class Class>
std::string Component<>::get(const Http::Uri& uri){
    Id id = uri.path(1);
    if(id.length()>0){
        Class* component = obtain<Class>(id);
        if(component) return component->get();
        else return Format(R"({"error":"id not found for type: %s, %s"})")<<Class::type()<<id;
    } else {
        std::string list = R"({"items":[)";
        for(auto component : filter<Class>()){
            list += Format(R"({"id":"%s"},)")<<component->id();
        }
        if(list.at(list.length()-1)==',') list.erase(list.end()-1);
        return list+"]}";
    }
}

template<class Class>
std::string Component<>::put(const Http::Uri& uri, const std::string& in){
    Id id = uri.path(1);
    Class* component = obtain<Class>(id);
    if(component) return component->put(in);
    else return Format(R"({"error":"id not found for type: %s, %s"})")<<Class::type()<<id;
}

template<class Class>
std::string Component<>::del(const Http::Uri& uri){
    return "{}";
}

}
