#include <stencila/component.hpp>

namespace Stencila {

Component::Component(void):
    meta_(nullptr) {}

Component::Component(const Component& other):
    meta_(nullptr) {}

Component::Component(const std::string& address) {
    initialise(address);
}

Component::~Component(void) {
    delete meta_;
}

}
