#pragma once

#include <stencila/component.hpp>

namespace Stencila {

class Package : public Component<Package> {
private:

    friend class Component<Package>;

    const char* type_(void) const {
    	return "package";
    }

    void read_(void){
    }
    
    void write_(void){
    }

public:

};

}