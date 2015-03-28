#include <stencila/hub.hpp>
#include <stencila/component.hpp>

using namespace Stencila;

#include "extension.hpp"

STENCILA_R_FUNC hub_signin(SEXP username, SEXP password){
    STENCILA_R_BEGIN
        hub.signin(
            as<std::string>(username),
            as<std::string>(password)
        );
    STENCILA_R_END
}

STENCILA_R_FUNC hub_username(void){
    STENCILA_R_BEGIN
        return wrap(hub.username());
    STENCILA_R_END
}

STENCILA_R_FUNC hub_signout(void){
    STENCILA_R_BEGIN
        hub.signout();
    STENCILA_R_END
}
