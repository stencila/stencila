#include <stencila/string.hpp>
#include <stencila/wamp.hpp>

namespace Stencila {
namespace Wamp {

Message::Message(void):
    Json::Document(Json::Array()) {
    append(int(NONE)); 
}

Message::Message(Type type):
    Json::Document(Json::Array()) {
    append(int(type));
}

Message::Message(const std::string& message, Type type):
    Json::Document(message) {

    int items = size();
    if(items<1) STENCILA_THROW(Exception,"Malformed WAMP message.\n  message: " + message);

    if(type != NONE) {
        char code = (*this)[code_].as<int>();
        if(code!=type) STENCILA_THROW(Exception,"Mismatched message code.\n  expected: " + string(type) + "\n  got: " + string(code)); 
    } 
}

Message::Type Message::type(void) const {
    return Type((*this)[code_].as<int>());
}


Call::Call(const std::string& message):
    Message(message, CALL) {

    int items = size();
    if(items<4) STENCILA_THROW(Exception,"Malformed CALL message.\n  message: " + message);
}

int Call::request(void) const {
    return (*this)[request_].as<int>();
}

std::string Call::procedure(void) const {
    return (*this)[procedure_].as<std::string>();
}

Json::Document Call::args(void) const {
    if (size()<=args_) STENCILA_THROW(Exception, "No arguments supplied");
    return (*this)[args_];
}

Message Call::result(const Json::Document& value) const {
    Message result(RESULT);
    result.append(request());
    Json::Document details = Json::Object();
    result.append(details);
    Json::Document yield_args = Json::Array();
    yield_args.append(value);
    result.append(yield_args);
    return result;
}

Message Call::error(const std::string& details) const {
    Message error(ERROR);
    error.append(request());
    error.append(details);
    return error;
}

}
}
