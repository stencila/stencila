#pragma once

#include <stencila/component.hpp>

namespace Stencila {

class Package : public Component {
public:

    std::string type(void) const {
    	return "package";
    }

};

}