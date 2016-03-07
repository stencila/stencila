#pragma once

#include <string>

#include <stencila/json.hpp>

namespace Stencila {
namespace Wamp {

/**
 * Very partial implementation of the WAMP protocol
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

    /**
     * Part identifiers
     */
    enum {
        code_ = 0
    };

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
};


/**
 * A remote procedure call
 * 
 * [CALL, Request|id, Options|dict, Procedure|uri]
 * [CALL, Request|id, Options|dict, Procedure|uri, Arguments|list]
 * [CALL, Request|id, Options|dict, Procedure|uri, Arguments|list, ArgumentsKw|dict]
 */
class Call : public Message {
 public:

    Call(const std::string& source);

    /**
     * Part identifiers
     */
    enum {
        request_ = 1,
        options_ = 2,
        procedure_ = 3,
        args_ = 4,
        kwargs_ = 5
    };

    /**
     * Get the request id
     */
    int request(void) const;

    /**
     * Get the procedure name
     */
    std::string procedure(void) const;

    /**
     * Get the list of arguments
     */
    Json::Document args(void) const;

    /**
     * Return a result
     *
     * The returned result includes the request id as per the specs:
     * 
     * [RESULT, CALL.Request|id, Details|dict]
     * [RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list]
     * [RESULT, CALL.Request|id, Details|dict, YIELD.Arguments|list, YIELD.ArgumentsKw|dict]
     */
    Message result(const Json::Document& result) const;

    /**
     * Return an error
     *
     * The returned error includes the request id as per the specs:
     * 
     * [ERROR, CALL, CALL.Request|id, Details|dict, Error|uri]
     * [ERROR, CALL, CALL.Request|id, Details|dict, Error|uri, Arguments|list]
     * [ERROR, CALL, CALL.Request|id, Details|dict, Error|uri, Arguments|list, ArgumentsKw|dict]
     */
    Message error(const std::string& details) const;
};

}
}
