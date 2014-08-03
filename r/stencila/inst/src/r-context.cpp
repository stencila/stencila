#include "r-context.hpp"

namespace Stencila {

RInside RContext::r_(
    0,{}, // argc and argv
    true, // loadRcpp (overidden to true in code anyway)
    false, // verbose
    true // interactive
);

unsigned int RContext::contexts_ = 0;

}