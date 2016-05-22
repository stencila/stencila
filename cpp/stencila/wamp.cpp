#include <array>

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
        char code = (*this)[MESSAGE_TYPE].as<int>();
        if(code!=type) STENCILA_THROW(Exception,"Mismatched message code.\n  expected: " + string(type) + "\n  got: " + string(code)); 
    } 
}

Message::Type Message::type(void) const {
    return Type((*this)[MESSAGE_TYPE].as<int>());
}

int Message::request(void) const {
    return (*this)[CALL_REQUEST].as<int>();
}

std::string Message::procedure(void) const {
    return (*this)[CALL_PROCEDURE].as<std::string>();
}

std::array<std::string, 2> Message::procedure_split(void) const {
    auto proc = procedure();
    int pos = proc.find('@');
    if (pos>=0){
        return {
            proc.substr(0,pos),
            proc.substr(pos+1)
        };
    } else {
        return {
            proc,
            ""
        };
    }
}

std::string Message::procedure_address(void) const {
    return procedure_split()[0];
}

std::string Message::procedure_method(void) const {
    return procedure_split()[1];
}

Json::Document Message::args(void) const {
    if (size()<=CALL_ARGS) STENCILA_THROW(Exception, "No arguments supplied");
    return (*this)[CALL_ARGS];
}

Json::Document Message::kwargs(void) const {
    if (size()<=CALL_KWARGS) STENCILA_THROW(Exception, "No keyword arguments supplied");
    return (*this)[CALL_KWARGS];
}

Message Message::result(const Json::Document& value) const {
    Message result(RESULT);
    result.append(request());
    Json::Document details = Json::Object();
    result.append(details);
    Json::Document yield_args = Json::Array();
    yield_args.append(value);
    result.append(yield_args);
    return result;
}

Message Message::error(const std::string& uri) const {
    Message error(ERROR);
    error.append(int(type()));
    error.append(request());
    Json::Document details = Json::Object();
    error.append(details);
    error.append(uri);
    return error;
}

}
}
