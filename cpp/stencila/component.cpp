//! @file component.cpp
//! @brief Implementation of a component static members
//! @author Nokome Bentley

#include <stencila/component.hpp>
#include <stencila/stencils.hpp>
#include <stencila/theme.hpp>
#include <stencila/simple-workspace.hpp>

namespace Stencila {

boost::uuids::random_generator Id::generator_;
std::map<Id,Component<>::Pointer> Component<>::pointers_;
std::map<std::string,Component<>::Type> Component<>::types_;

void Component<>::declarations(void){
    Component<>::declare<Theme>();
    Component<>::declare<Stencils::Stencil>();
    Component<>::declare<SimpleWorkspace>();
}

}
