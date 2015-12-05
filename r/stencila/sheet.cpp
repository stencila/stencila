#include <stencila/sheet.hpp>
using namespace Stencila;

#include "stencila.hpp"

STENCILA_R_NEW(Sheet)

STENCILA_R_EXEC1(Sheet,initialise,std::string)

STENCILA_R_EXEC2(Sheet,load,std::string,std::string)
STENCILA_R_RET1(Sheet,dump,std::string)

STENCILA_R_EXEC1(Sheet,import,std::string)
// Need to wrap the `export` method manually
// because `export` is a keyword in C++
STENCILA_R_FUNC Sheet_export(SEXP self,SEXP path){
    STENCILA_R_BEGIN
        from<Sheet>(self).export_(as<std::string>(path));
        return null;
    STENCILA_R_END
}

STENCILA_R_EXEC1(Sheet,read,std::string)
STENCILA_R_EXEC1(Sheet,write,std::string)

STENCILA_R_EXEC0(Sheet,compile)
