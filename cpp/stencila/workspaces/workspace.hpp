//! @file workspace.hpp
//! @brief Definition of class Workspace
//! @author Nokome Bentley

#pragma once

#include <string>

#include <stencila/component.hpp>

namespace Stencila {
namespace Workspaces {

template<class Class>
class Workspace : public Component<Class> {
protected:

    typedef std::string String;

public:

    static String type(void){
        return "workspace";
    };
    
    //! @name Persistence methods
    //! @{
    
    //! @}
    
    //! @name REST interface methods
    //! @{

    String post(const String& method, const Http::Uri& uri, const String& data){
        Class* self = static_cast<Class*>(this);
        if(method=="interact"){
            Json::Document json(data);
            if(json.has("code")) {
                String code = json.as<String>(json.get("code"));
                String result = self->interact(code);
                return Format(R"({"return":"%s"})")<<result;
            }
            else return R"({"error":"required field missing:'code'"})";
        }
        else if(method=="set"){
            String name = uri.field("name");
            String expression = uri.field("expression");
            if(name.length()==0) return R"({"error":"required field missing:'name'"})";
            if(expression.length()==0) return R"({"error":"required field missing:'expression'"})";
            self->set(name,expression);
            return "{}";
        } else if(method=="text"){
            String expression = uri.field("expression");
            if(expression.length()==0) return R"({"error":"required field missing:'expression'"})";
            String text = self->text(expression);
            return Format(R"({"return":"%s"})")<<text;
        } else {
            return Format(R"({"error":"unknown method: %s"})")<<method;
        }
    }
    
    String get(void) {
        this->read();
        Json::Document out;
        return out.dump();
    }
    
    String put(const String& data){
        Json::Document json(data);
        this->write();
        return "{}";
    }
    
    //! @}

    //! @brief 
    //! @param name
    //! @param expression
    void set(const String& name, const String& expression){
    }

    //! @brief 
    //! @param code
    void script(const String& code){
    }
    
    //! @brief 
    //! @param code
    //! @return 
    String interact(const String& code){
        return "";
    }

    //! @brief 
    //! @param expression
    //! @return 
    String text(const String& expression){
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
}
