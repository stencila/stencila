#include <stencila/component.hpp>
#include <stencila/stencil.hpp>
#include <stencila/network.hpp>

namespace Stencila {

// It is important that classes are put into the classes_ array in the right
// order and that the thods are also in the right order!
Component::Class Component::classes_[Component::class_codes_];

std::map<std::string,Component::Instance> Component::instances_;

// Implemented here to prevent circular dependency in 
// `component.hpp` and `network.hpp`
std::string Component::serve(ClassCode code){
    // Declare this component
    declare(code);
    // Ensure the Server is started
    std::string url = Server::ensure();
    // Add this component's address to the url
    url += "/" + address();
    return url;
}

}
