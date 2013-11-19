//! @file simple-workspace.hpp
//! @brief Definition of class SimpleWorkspace
//! @author Nokome Bentley

#pragma once

#include <stencila/workspace.hpp>

namespace Stencila {

class SimpleWorkspace : public Workspace<SimpleWorkspace> {
private:
    typedef std::string String;
    typedef std::map<String,String> Map;
    
    Map map_;
    
public:

    static String type(void){
        return "simple-workspace";
    };
    
    SimpleWorkspace(void){
    }
    
    SimpleWorkspace(const Id& id){
    }

    //! @brief 
    //! @param name
    //! @param expression
    void set(const String& name, const String& expression){
        map_[name] = expression;
    }

    //! @brief 
    //! @param code
    void script(const String& code){
    }

    //! @brief 
    //! @param expression
    //! @return 
    String text(const String& expression){
        auto i = map_.find(expression);
        if(i!=map_.end()) return i->second;
        else return "";
    }

    void image_begin(const String& type){
    }

    String image_end(void){
        return "";
    }

    //! brief   
    //! @param expression
    //! @return 
    bool test(const String& expression){
        return false;
    }

    //! @brief 
    //! @param expression
    void subject(const String& expression){
    }

    //! @brief 
    //! @param expression
    //! @return 
    bool match(const String& expression){
        return false;
    }

    //! @brief 
    void enter(void){
    }
    
    //! @brief 
    //! @param expression
    void enter(const String& expression){
    }

    //! @brief 
    void exit(void){
    }

    //! @brief 
    //! @param item
    //! @param items
    //! @return 
    bool begin(const String& item,const String& items){
        return false;
    }

    //! @brief 
    //! @return 
    bool step(void){
        return false;
    }
    
};

}
