#include <stencila/theme.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_NEW(Theme)

STENCILA_R_EXEC1(Theme,initialise,std::string)

STENCILA_R_GET(Theme,title)
STENCILA_R_GET(Theme,description)
STENCILA_R_GET(Theme,keywords)
STENCILA_R_GET(Theme,authors)

STENCILA_R_GETSET(Theme,style,std::string)
STENCILA_R_GETSET(Theme,behaviour,std::string)

STENCILA_R_RET0(Theme,serve) 
STENCILA_R_EXEC0(Theme,view)

STENCILA_R_EXEC0(Theme,compile)
