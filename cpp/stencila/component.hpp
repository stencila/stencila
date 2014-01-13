#pragma once

#include <string>
#include <vector>

namespace Stencila {

template<class Class>
class Component {
    
private:
    std::string title_;
    std::string description_;  
    std::vector<std::string> keywords_;
    std::vector<std::string> authors_;

public:

    /**
     * @{
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
     * @param title Title of the component
     */
    Class& title(const std::string& title) {
        title_ = title;
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
     * @param description Description for the component
     */
    Class& description(const std::string& description) {
        description_ = description;
        return static_cast<Class&>(*this);
    }
    
    /**
     * Get component keywords
     */
    const std::vector<std::string>& keywords(void) const {
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
     * @param keywords Keywords for the component
     */
    Class& keywords(const std::vector<std::string>& keywords) {
        keywords_ = keywords;
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
     * @param authors Authors of the component
     */
    Class& authors(const std::vector<std::string>& authors) {
        authors_ = authors;
        return static_cast<Class&>(*this);
    }

    /**
     * @}
     */

 };

}
