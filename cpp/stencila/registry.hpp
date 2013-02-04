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

//! @file registry.hpp
//! @brief Definition of Registry which holds information on local stencila objects
//! @author Nokome Bentley

#pragma once

#include <string>
#include <map>

#include <boost/filesystem.hpp>
#include <boost/uuid/uuid.hpp>
#include <boost/uuid/uuid_generators.hpp>
#include <boost/uuid/uuid_io.hpp>
#include <boost/date_time/posix_time/posix_time.hpp>

namespace Stencila {

typedef std::string Id;

class Registry {
private:

    struct Item {
        std::string type;
        void* pointer;
    };
    
    std::map<Id,Item> items_;
    
public:
    Registry(void){
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
    
    static Id id(void) {
        // Generate a UUID
        boost::uuids::uuid uuid = boost::uuids::random_generator()();
        // Convert from chars to hex based on http://stackoverflow.com/a/69197/1583041
        // There may be better way to do this.
        unsigned char chars[16];
        std::memcpy(&uuid,chars,16);
        static char const* digits = "0123456789abcdef";
        std::string hex(32,0);
        std::string::iterator pos = hex.begin();
        for(int i=0;i<16;i++){
            unsigned char character = chars[i];
            *pos++ = digits[character>>4];
            *pos++ = digits[character&15];
        }
        return hex;
    }
    
    template<class Type>
    void set(const std::string& type, const Id& id, Type* instance){
        items_[id] = {type,instance};
    }
    
    template<class Type>
    Type* get(const std::string& type, const Id& id){
        auto i = items_.find(id);
        if(i!=items_.end()){
            if(type==i->second.type) return static_cast<Type*>(i->second.pointer);
            else return 0;
        }
        else return 0;
    }

} Registry;

}
