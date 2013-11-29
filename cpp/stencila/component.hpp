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
    std::string desc_;  
    std::vector<std::string> tags_;
    std::vector<std::string> authors_;

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
    //! @{ 
    
    struct Pointer {
        std::string type;
        Component<void>* pointer;
    };
    static std::map<Id,Pointer> pointers_;
    
public:
    
    static void record(const std::string& type,Component<void>* instance){
        pointers_[instance->id()] = {type,instance};
    }
    
    template<class Class>
    static Class* obtain(const Id& id){
        std::string type = Class::type();
        auto i = pointers_.find(id);
        if(i!=pointers_.end()){
            if(i->second.type==type) return static_cast<Class*>(i->second.pointer);
            else return 0;
        } else {
            std::string dir = directory(id);
            if(boost::filesystem::exists(dir)){
                Class* component = static_cast<Class*>(create<Class>(id));
                return component;
            } else {
                return 0;
            }
        }
    }
    
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

    /**
     * Get component id
     */
    const Id id(void) const {
        return id_;
    }

    static std::string directory(const Id& id) {
        boost::filesystem::path dir(home());
        dir /= "components";
        std::vector<std::string> list;
        boost::algorithm::split(list,id,boost::is_any_of("."));
        for(auto item : list) dir /= item;
        return dir.string();
    }
    
    std::string directory(void) const {
        return directory(id());
    }
    

    //! @brief Get the path to the user's Stencila directory which holds Stencila data.
    //!
    //! This is a first attempt at generating a cross platform home directory path. Note that on Windows
    //! and Mac, aplication data usually goes in specific directories, not the ".stencila" directory as is *nix convention
    //! See:
    //!     http://stackoverflow.com/questions/4891006/how-to-create-a-folder-in-the-home-directory
    //!     http://stackoverflow.com/questions/2552416/how-can-i-find-the-users-home-dir-in-a-cross-platform-manner-using-c
    //!     http://stackoverflow.com/questions/2910377/get-home-directory-in-linux-c
    //! @return Path to the user's Stencila directory
    static std::string home(void) {
        std::string home = std::getenv("HOME");
        if(not home.length()) {
            home = std::getenv("USERPROFILE");
        }
        if(not home.length()) {
            std::string home_drive = std::getenv("HOMEDRIVE");
            std::string home_path = std::getenv("HOMEPATH");
            home = home_drive+home_path;
        }
        if(not home.length()) {
            home = boost::filesystem::current_path().string();
        }
        return home + "/.stencila/";
    }
    
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
     * Get component description
     */
    const std::string desc(void) const {
        return desc;
    }

    /**
     * Set component description
     */
    Class& desc(const std::string& value) {
        desc_ = value;
        return static_cast<Class&>(*this);
    }
    
    /**
     * Get component tags
     */
    const std::vector<std::string> tags(void) const {
        return tags_;
    }

    /**
     * Get component tags
     */
    std::vector<std::string>& tags(void) {
        return tags_;
    }

    /**
     * Set component tags
     */
    Class& tags(const std::vector<std::string>& values) {
        tags_ = values;
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
     * Read the component from the it's default directory
     */
    Class& read(void){
        std::string dir = directory();
        if(boost::filesystem::exists(dir)){
            static_cast<Class*>(this)->read(dir);
        } else {
            STENCILA_THROW(Exception,"Directory does not exist: "+dir);
        }
        return static_cast<Class&>(*this);
    }
    
    /**
     * Read the Component from a directory
     *
     * This method should be overidden by component classes
     * 
     * @param directory Filesystem path to directory
     */
    Class& read(const std::string& directory){
        STENCILA_THROW(Unimplemented,"Component<Class>::read");
        return static_cast<Class&>(*this);
    }
    
    /**
     * Write the Component to it's default directory
     */
    Class& write(void) {
        std::string dir = directory();
        boost::filesystem::create_directories(dir);
        static_cast<Class*>(this)->write(dir);
        return static_cast<Class&>(*this);
    }
    
    /**
     * Write the Component to a directory
     *
     * This method should be overidden by component classes
     * 
     * @param directory Filesystem path to directory
     */
    Class& write(const std::string& directory){
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
