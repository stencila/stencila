#pragma once

#include <string>

#include <stencila/json.hpp>

namespace Stencila {
namespace Wamp {

/**
 * A WAMP message
 *
 * Currently only partially implemented focussing on the Remote Procedure Call (RPC)
 * aspects.
 *
 * https://tools.ietf.org/html/draft-oberstet-hybi-tavendo-wamp-02
 */
class Message : public Json::Document {
 public:

    /**
     * Message type codes
     */
    enum Type {
        NONE = 0,

        HELLO = 1,
        WELCOME = 2,
        ABORT = 3,
        GOODBYE = 6,
        
        ERROR = 8,

        PUBLISH = 16,
        PUBLISHED = 17,

        SUBSCRIBE = 32,
        SUBSCRIBED = 33,
        UNSUBSCRIBE = 34,
        UNSUBSCRIBED = 35,
        EVENT = 36,

        CALL = 48,
        RESULT = 50,

        REGISTER = 64,
        REGISTERED = 65,
        UNREGISTER = 66,
        UNREGISTERED = 67,
        INVOCATION = 68,
        YIELD = 70
    };

    static const char MESSAGE_TYPE = 0;

    /**
     * [CALL, Request|id, Options|dict, Procedure|uri]
     * [CALL, Request|id, Options|dict, Procedure|uri, Arguments|list]
     * [CALL, Request|id, Options|dict, Procedure|uri, Arguments|list, ArgumentsKw|dict]
     */
    static const char CALL_REQUEST = 1;
    static const char CALL_OPTIONS = 2;
    static const char CALL_PROCEDURE = 3;
    static const char CALL_ARGS = 4;
    static const char CALL_KWARGS = 5;

    /**
     * [RESULT, CALL.Request|id, Details|dict]
     * [RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
     * [RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list, YIELD.ArgumentsKw|dict]
     */
    static const char RESULT_REQUEST = 1;
    static const char RESULT_DETAILS = 2;
    static const char RESULT_ARGS_ = 3;
    static const char RESULT_KWARGS = 4;
        
    /**
     * [ERROR, CALL, CALL.Request|id, Details|dict, Error|uri]
     * [ERROR, CALL, CALL.Request|id, Details|dict, Error|uri, Arguments|list]
     * [ERROR, CALL, CALL.Request|id, Details|dict, Error|uri, Arguments|list, ArgumentsKw|dict]
     */
    static const char ERROR_TYPE = 1;
    static const char ERROR_REQUEST = 2;
    static const char ERROR_DETAILS = 3;
    static const char ERROR_URI = 4;
    static const char ERROR_ARGS = 5;
    static const char ERROR_KWARGS = 6;

    /**
     * Constructors
     */
    
    Message(void);

    Message(Type type);

    Message(const std::string& message, Type type = NONE);

    /**
     * Get the type of this message
     */
    Type type(void) const;

    /**
     * Get the request id
     */
    int request(void) const;

    /**
     * Get the procedure identifier
     */
    std::string procedure(void) const;

    /**
     * Split the procedure identifier into address and method parts
     *
     * e.g. `demo/sheets/iris@update` : {`demo/sheets/iris`,`update`}
     */
    std::array<std::string, 2> procedure_split(void) const;

    /**
     * Get the address part of the procedure identifier
     */
    std::string procedure_address(void) const;

    /**
     * Get the method part of the procedure identifier
     */
    std::string procedure_method(void) const;

    /**
     * Get the argument array
     */
    Json::Document args(void) const;

    /**
     * Get the keyword argument object
     */
    Json::Document kwargs(void) const;

    /**
     * Generate a result message
     */
    Message result(const Json::Document& result) const;

    /**
     * Generate a error message
     */
    Message error(const std::string& details) const;
};

}
}
