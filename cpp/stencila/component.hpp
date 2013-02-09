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

//! @file component.hpp
//! @brief Definition of class Component
//! @author Nokome Bentley

#pragma once

#include <string>
#include <map>

#include <boost/lexical_cast.hpp>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>

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
        // Generate a UUID
        // Convert from chars to hex based on http://stackoverflow.com/a/69197/1583041
        // There may be better way to do this.
        boost::uuids::uuid uuid = generator_();
        unsigned char chars[16];
        std::memcpy(chars,&uuid,16);
        static char const* digits = "0123456789abcdef";
        std::string hex(32,0);
        std::string::iterator pos = hex.begin();
        for(int i=0;i<16;i++){
            unsigned char character = chars[i];
            *pos++ = digits[character>>4];
            *pos++ = digits[character&15];
        }
        assign("anon."+hex);
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
    
    struct Registration {
        std::string type;
        Component<void>* pointer;
    };
    static std::map<Id,Registration> registry_;
    
    static void record(const std::string& type,Component<void>* instance){
        registry_[instance->id()] = {type,instance};
    }

public:

    Component(const std::string& type):
        id_(){
        record(type,this);
    }
    
    Component(const std::string& type,const Id& id):
        id_(id){
        record(type,this);
    }
    
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
    static Class& create(void);
    
    template<class Class>
    static Class& create(const Id& id);
    
    //! @}
    
    
    //! @name Component retrieval methods
    //! @{
    
    template<class Class>
    static Class* obtain(const Id& id);
    
    template<class Class>
    static std::vector<Class*> filter(void);
    
    //! @}
    
    
    //! @name REST methods
    //! @{
    
    static std::string rest(const std::string& method, const std::string& uri, const std::string& json);
    
    static std::string rest(const Http::Method& verb, const Http::Uri& uri, const std::string& json);
    
    template<class Class>
    static std::string rest_type(const Http::Method& verb, const Http::Uri& uri, const std::string& json);

    template<class Class>
    static std::string post(const Http::Uri& uri, const std::string& in);
    
    template<class Class>
    static std::string get(const Http::Uri& uri);
    
    template<class Class>
    static std::string put(const Http::Uri& uri, const std::string& in);
    
    template<class Class>
    static std::string del(const Http::Uri& uri);
    
    //! @}
};


template<class Class>
class Component : public Component<void> {
public:

    Component<Class>(void):
        Component<void>(Class::type()){
    }

    Component<Class>(const Id& id):
        Component<void>(Class::type(),id){
    }
};

}
