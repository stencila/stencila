#pragma once

#include <string>
#include <vector>

namespace Stencila {

/**
 * Component
 */
template<class Class>
class Component {
    
private:

    /**
     * Component information
     *
     * Encapsulated as a separate class to reduce the minimum size of a 
     * Component object to the sizeof(void*). 
     */
    class Info {
    public:
        std::string title;
        std::string description;  
        std::vector<std::string> keywords;
        std::vector<std::string> authors;
    };

    /**
     * Information on the component
     *
     * Only created if an Info attribute is accessed
     * for this component.
     */
    Info* info_;

public:

    Component(void):
        info_(nullptr){
    }

    ~Component(void){
        if(info_) delete info_;
    }

    /**
     * @{
     * @name Info attribute getters and setters
     */

    //Define some local get and set macros
    
    #define _GET(attr,value) \
        if(not info_){ \
            info_ = new Info; \
            info_->attr = value; \
        } \
        return info_->attr;

    #define _SET(attr,value) \
        if(not info_) info_ = new Info; \
        info_->attr = value; \
        return static_cast<Class&>(*this);

    /**
     * Get component title
     */
    const std::string title(void) const {
        _GET(title,"")
    }

    /**
     * Get component title
     */
    std::string& title(void) {
        _GET(title,"")
    }

    /**
     * Set component title
     * @param title Title of the component
     */
    Class& title(const std::string& title) {
        _SET(title,title)
    }

    /**
     * Get component description
     */
    const std::string& description(void) const {
        _GET(description,"")
    }

    /**
     * Get component description
     */
    std::string& description(void) {
        _GET(description,"")
    }

    /**
     * Set component description
     * @param description Description for the component
     */
    Class& description(const std::string& description) {
        _SET(description,description)
    }
    
    /**
     * Get component keywords
     */
    const std::vector<std::string>& keywords(void) const {
        _GET(keywords,std::vector<std::string>(0))
    }

    /**
     * Get component keywords
     */
    std::vector<std::string>& keywords(void) {
        _GET(keywords,std::vector<std::string>(0))
    }

    /**
     * Set component keywords
     * @param keywords Keywords for the component
     */
    Class& keywords(const std::vector<std::string>& keywords) {
        _SET(keywords,keywords)
    }

    /**
     * Get component authors
     */
    const std::vector<std::string>& authors(void) const {
        _GET(authors,std::vector<std::string>(0))
    }

    /**
     * Get component authors
     */
    std::vector<std::string>& authors(void) {
        _GET(authors,std::vector<std::string>(0))
    }

    /**
     * Set component authors
     * @param authors Authors of the component
     */
    Class& authors(const std::vector<std::string>& authors) {
        _SET(authors,authors)
    }

    // Undefine local macros
    #undef _GET
    #undef _SET

    /**
     * @}
     */

 };

}
