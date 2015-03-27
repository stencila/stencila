#include <stencila/theme.hpp>
using namespace Stencila;

#include "extension.hpp"

STENCILA_R_NEW(Theme)

STENCILA_R_EXEC1(Theme,initialise,std::string)

STENCILA_R_GETSET(Theme,style,std::string)
STENCILA_R_GETSET(Theme,behaviour,std::string)

STENCILA_R_EXEC0(Theme,compile)

STENCILA_R_RET0(Theme,serve) 
STENCILA_R_EXEC0(Theme,view)
