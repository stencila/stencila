#include <stencila/tables/cursor.hpp>
using namespace Stencila::Tables;

#include "r-extension.hpp"

STENCILA_R_FUNC Cursor_fetch(SEXP self){
    STENCILA_R_BEGIN
        return wrap(
            from<Cursor>(self).fetch()
        );
    STENCILA_R_END
}