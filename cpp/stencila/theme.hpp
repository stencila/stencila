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
    
    void read_from(const std::string& directory){
        std::ifstream file(directory+"/style.less");
        std::string value((std::istreambuf_iterator<char>(file)),(std::istreambuf_iterator<char>()));
        style(value);
    }
    
    void write_to(const std::string& directory) {
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
