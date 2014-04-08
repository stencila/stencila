#include <stencila/component.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_NEW(Component)

STENCILA_R_ATTR(Component,title,std::string)
STENCILA_R_ATTR(Component,description,std::string)
STENCILA_R_ATTR(Component,keywords,std::vector<std::string>)
STENCILA_R_ATTR(Component,authors,std::vector<std::string>)

STENCILA_R_ATTR(Component,path,std::string)

STENCILA_R_EXEC1(Component,commit,std::string)

