//! @file component.cpp
//! @brief Implementation of component static members
//! @author Nokome Bentley

#include <stencila/component.hpp>
#include <stencila/stencils.hpp>
#include <stencila/workspaces.hpp>
#include <stencila/theme.hpp>

namespace Stencila {

boost::uuids::random_generator Id::generator_;
std::map<Id,Component<>::Pointer> Component<>::pointers_;
std::map<std::string,Component<>::Type> Component<>::types_;

void Component<>::declarations(void){
    Component<>::declare<Stencils::Stencil>();
    Component<>::declare<Workspaces::Simple>();
    Component<>::declare<Theme>();
}

}
