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
