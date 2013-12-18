//! @file component.hpp
//! @brief Definition of class Component
//! @author Nokome Bentley

#pragma once

#include <string>
#include <sstream>
#include <map>

#include <boost/lexical_cast.hpp>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/archive/iterators/base64_from_binary.hpp>
#include <boost/archive/iterators/transform_width.hpp>
#include <boost/archive/iterators/ostream_iterator.hpp>
#include <boost/algorithm/string/replace.hpp>

#include <stencila/system.hpp>
#include <stencila/http.hpp>
#include <stencila/json.hpp>
#include <stencila/format.hpp>

namespace Stencila {

template<class Type=void> class Component;

class Id : public std::string {
private:
    static boost::uuids::random_generator generator_;

public:
    Id(void){
        // Generate a UUID using boost::uuids
        boost::uuids::uuid uuid = generator_();
        unsigned char chars[16];
        std::memcpy(chars,&uuid,16);

        // Transform characters to Base^4 encoding
        // Based on http://stackoverflow.com/a/7053808
        using namespace boost::archive::iterators;
        std::stringstream stringstream;
        typedef base64_from_binary<
                    transform_width<const char*,6,8>
                >  base64_text;
        std::copy(
            base64_text(chars),
            base64_text(chars+16),
            ostream_iterator<char>(stringstream)
        );
        std::string base64 = stringstream.str();

        // Replace the two Base64 characters which are unsafe or reserved in URL encoding
        // See for example http://perishablepress.com/stop-using-unsafe-characters-in-urls/
        // Replace '+' which is a 'safe' but 'reserved' character
        boost::replace_all(base64,"+",".");
        // Replace '/' which is 'reserved'
        boost::replace_all(base64,"/","_");

        //Assign to self
        assign(base64);
    }
    
    Id(const std::string& id):
        std::string(id){
    }
    
    Id& operator=(const std::string& value){
        assign(value);
        return *this;
    }
};

template<>
class Component<void> {
protected:

    Id id_;

public:

    /**
     * @name Constructors
     * @{
     */
    
    Component(const std::string& type):
        id_(){
        record(type,this);
    }
    
    Component(const std::string& type,const Id& id):
        id_(id){
        record(type,this);
    }
    
    /**
     * @}
     */
    
    //! @name Component creation methods
    //! @{
    
    template<class Class>
    static Component<>* create(void){
        return new Class;
    }
    
    template<class Class>
    static Component<>* create(const Id& id){
        return new Class(id);
    }
    
    //! @}

    /**
     * Get component id
     */
    const Id id(void) const {
        return id_;
    }


    //! @name Component type declaration and definition methods
    //! @{
    
protected:

    struct Type {
        bool defined;
        typedef std::string (*RestMethod)(const Http::Method& verb, const Http::Uri& uri, const std::string& json);
        RestMethod rest;
    };
    static std::map<std::string,Type> types_;
    
public:

    static std::string type(void){
        return "component";
    };

    template<class Class>
    static void declare(void){
        Type type {
            true,
            static_cast<Type::RestMethod>(&rest_type<Class>)
        };
        types_[Class::type()] = type;
    }

    static void declarations(void);

    static Type definition(std::string type){ 
        auto i = types_.find(type);
        if(i!=types_.end()) return i->second;
        else return Type {false};
    }

    //! @}

protected:
    
    //! @name Component retrieval methods
    //! 
    //! These methods allow the registration and retreival of those components
    //! that have been consturcted in memory
    //! 
    //! @{ 
    
    struct Pointer {
        std::string type;
        Component<void>* pointer;
    };
    static std::map<Id,Pointer> pointers_;
    
    /**
     * Register a component instance
     * @param type     String representing type of component
     * @param instance Component pointer
     */
    static void record(const std::string& type,Component<void>* instance){
        pointers_[instance->id()] = {type,instance};
    }
    
public:

    /**
     * Get a pointer to the component with a given id
     * @param id Identifier
     */
    template<class Class>
    static Class* obtain(const Id& id){
        std::string type = Class::type();
        auto i = pointers_.find(id);
        if(i!=pointers_.end()){
            if(i->second.type==type) return static_cast<Class*>(i->second.pointer);
            else return 0;
        }
    }
    
    /**
     * Get a vector of pointers to all components of a particular class
     */
    template<class Class>
    static std::vector<Class*> filter(void){
        std::string type = Class::type();
        std::vector<Class*> filtered;
        for(const std::map<Id,Pointer>::value_type& i : pointers_){
            if(i.second.type==type) filtered.push_back(static_cast<Class*>(i.second.pointer));
        }
        return filtered;
    }
    
    //! @}

    //! @name Component loading methods
    //! 
    //! These methods are for finding component on the disk or the network
    //! 
    //! @{ 


    //! @}

    
    //! @name REST methods
    //! @{
    
    static std::string rest(const std::string& method, const std::string& uri, const std::string& json){
        return rest(Http::Method(method),Http::Uri(uri),json);
    }
    
    static std::string rest(const Http::Method& verb, const Http::Uri& uri, const std::string& json){
        try{
            std::string type_name = uri.segment(0);
            if(type_name.length()==0) return R"({"error":"type not specified"})";
            Type type = definition(type_name);
            if(type.defined) return type.rest(verb,uri,json);
            else return Format(R"({"error":"undefined type: %s"})")<<type_name;
        } catch (std::exception &e) {
            return Format(R"({"error":"%s"})")<<e.what();
        } catch (...) {
            return R"({error:unknown})";
        }
    }

    template<class Class>
    static std::string rest_type(const Http::Method& verb, const Http::Uri& uri, const std::string& json){
        if(verb==Http::Post) return post<Class>(uri,json);
        else if(verb==Http::Get) return get<Class>(uri);
        else if(verb==Http::Put) return put<Class>(uri,json);
        else if(verb==Http::Delete) return del<Class>(uri);
        else return Format(R"({"error":"unsupported method: %s"})")<<verb;
    }

    template<class Class>
    static std::string post(const Http::Uri& uri, const std::string& json){
        Id id = uri.segment(1);
        if(id.length()==0){
            Class* component = static_cast<Class*>(create<Class>());
            component->put(json);
            return Format(R"({"id":"%s"})")<<component->id();
        } else {
            Class* component = obtain<Class>(id);
            if(component){
                std::string method = uri.segment(2);
                if(method.length()>0) return component->post(method,uri,json);
                else return Format(R"({"error":"method must be given when POSTing with id"})");
            } else return Format(R"({"error":"id not found for type: %s, %s"})")<<Class::type()<<id;
        }
    }

    template<class Class>
    static std::string get(const Http::Uri& uri){
        Id id = uri.segment(1);
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
    static std::string put(const Http::Uri& uri, const std::string& in){
        Id id = uri.segment(1);
        Class* component = obtain<Class>(id);
        if(component) return component->put(in);
        else return Format(R"({"error":"id not found for type: %s, %s"})")<<Class::type()<<id;
    }

    template<class Class>
    static std::string del(const Http::Uri& uri){
        return R"({"error":"DELETE not yet implemented"})";
    }

    //! @}
};

template<class Class>
class Component : public Component<> {
protected:
    std::string title_;
    std::string description_;  
    std::vector<std::string> keywords_;
    std::vector<std::string> authors_;

public:

    Component<Class>(void):
        Component<>(Class::type()){
    }

    Component<Class>(const Id& id):
        Component<>(Class::type(),id){
    }


    /**
     * @name Attribute getters and setters
     */

    /**
     * Get component title
     */
    const std::string& title(void) const {
        return title_;
    }

    /**
     * Get component title
     */
    std::string& title(void) {
        return title_;
    }

    /**
     * Set component title
     */
    Class& title(const std::string& value) {
        title_ = value;
        return static_cast<Class&>(*this);
    }

    /**
     * Get component description
     */
    const std::string& description(void) const {
        return description_;
    }

    /**
     * Get component description
     */
    std::string& description(void) {
        return description_;
    }

    /**
     * Set component description
     */
    Class& descriptionc(const std::string& value) {
        description_ = value;
        return static_cast<Class&>(*this);
    }
    
    /**
     * Get component keywords
     */
    const std::vector<std::string> keywords(void) const {
        return keywords_;
    }

    /**
     * Get component keywords
     */
    std::vector<std::string>& keywords(void) {
        return keywords_;
    }

    /**
     * Set component keywords
     */
    Class& keywords(const std::vector<std::string>& values) {
        keywords_ = values;
        return static_cast<Class&>(*this);
    }

    /**
     * Get component authors
     */
    const std::vector<std::string> authors(void) const {
        return authors_;
    }

    /**
     * Get component authors
     */
    std::vector<std::string>& authors(void) {
        return authors_;
    }

    /**
     * Set component authors
     */
    Class& authors(const std::vector<std::string>& values) {
        authors_ = values;
        return static_cast<Class&>(*this);
    }

    
    //! @name Persistence methods
    //! @{
    
    /**
     * Find the location of a Component matching name
     * @param  name Name of directory or file root to search for
     * @return      Path to the component
     */
    std::string find(const std::string& name){

    }
    
    /**
     * Read the component
     */
    Class& read(std::string name=""){
        if(name.length()==0) name = id();
        std::string location = find(name);
        static_cast<Class*>(this)->read(location,name);
        return static_cast<Class&>(*this);
    }
    
    /**
     * Read the Component from a directory
     *
     * This method should be overidden by component classes
     * 
     * @param directory Filesystem path to directory
     * @param name Name for component files
     */
    Class& read(const std::string& directory,const std::string& name){
        STENCILA_THROW(Unimplemented,"Component<Class>::read");
        return static_cast<Class&>(*this);
    }
    
    /**
     * Write the Component to the default directory
     */
    Class& write(void) {
        static_cast<Class*>(this)->write(home()+"/components",id);
        return static_cast<Class&>(*this);
    }
    
    /**
     * Write the Component to a directory
     *
     * This method should be overidden by component classes
     * 
     * @param directory Filesystem path to directory
     * @param name Name for component files
     */
    Class& write(const std::string& directory,const std::string& name){
        STENCILA_THROW(Unimplemented,"Component<Class>::write");
        return static_cast<Class&>(*this);
    }
    
    //! @}
    
    //! @name REST methods
    //! @{
    
    std::string post(const std::string& method, const Http::Uri& uri, const std::string& data){
        return "{}";
    }
    
    std::string get(void) {
        return "{}";
    }
    
    std::string put(const std::string& data){
        return "{}";
    }
    
    //! @}
};

}
