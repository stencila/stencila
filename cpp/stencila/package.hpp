#pragma once

#include <stencila/component.hpp>

namespace Stencila {

class Package : public Component<Package> {
public:

    std::string type(void) const {
    	return "package";
    }

};

}