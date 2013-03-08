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

//! @file theme.hpp
//! @brief Definition of class Theme
//! @author Nokome Bentley

#pragma once

#include <string>
#include <fstream>

#include <boost/filesystem.hpp>

#include <stencila/component.hpp>
#include <stencila/json.hpp>

namespace Stencila {

class Theme : public Component<Theme> {
private:

    std::string style_;

public:

    static std::string type(void){
        return "theme";
    }
    
    Theme(void):
        Component<Theme>(){
    }
    
    Theme(const Id& id):
        Component<Theme>(id){
        read();
    }
    
    std::string style(void) const {
        return style_;
    }
    
    Theme& style(const std::string& style){
        style_ = style;
        return *this;
    }

    //! @name Persistence methods
    //! @{
    
    void read_from(const String& directory){
        std::ifstream file(directory+"/style.less");
        std::string value((std::istreambuf_iterator<char>(file)),(std::istreambuf_iterator<char>()));
        style(value);
    }
    
    void write_to(const String& directory) {
        std::ofstream file(directory+"/style.less");
        file<<style();
    }
    
    //! @}
    
    //! @name REST interface methods
    //! @{

    std::string get(void) {
        read();
        Json::Document out;
        out.add("style",style_);
        return out.dump();
    }
    
    std::string put(const std::string& data){
        Json::Document json(data);
        if(json.has("style")){
            style_ = json.as<std::string>(json.get("style"));
        }
        write();
        return "{}";
    }
    
    //! @}
    
};

}
