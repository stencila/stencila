#pragma once

#include <iostream>
#include <vector>

#include <boost/algorithm/string.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Syntax {

struct Node {
    virtual ~Node(void) {
    }
};

struct Boolean : Node {
    bool value;

    explicit Boolean(const char* string) {
        std::string v = string;
        boost::algorithm::to_lower(v);
        if (v == "true") {
            value = true;
        } else if (v == "false") {
            value = false;
        } else {
            STENCILA_THROW(Exception, "Invalid string value for Boolean.\n  value: " + value);
        }
    }

    ~Boolean(void){
    }
};

struct Number : Node {
    std::string value;

    explicit Number(const char* value):
        value(value) {}

    ~Number(void){
    }
};

struct String : Node {
    std::string value;

    explicit String(const char* value):
        value(value) {}

    ~String(void){
    }
};

struct Identifier : Node {
    std::string value;

    explicit Identifier(const char* value):
        value(value) {}

    ~Identifier(void){
    }
};

struct Range : Node {
    Node* first;
    Node* last;

    Range(Node* first, Node* last):
        first(first),
        last(last) {}

    ~Range(void){
    }
};

struct Binary : Node {
    char symbol;
    Node* left;
    Node* right;

    Binary(char symbol, Node* left, Node* right):
        symbol(symbol),
        left(left),
        right(right) {}

    ~Binary(void){
        delete left;
        delete right;
    }
};

struct Call : Node {
    std::string function;
    std::vector<Node*> arguments;

    /**
     * Are the arguments borrowed from another Call?
     *
     * When constructing a Call using the arguments of another Call
     * borrowed must be set to `true` so that the arguments are not 
     * deleted on destruction of this call.
     */
    bool borrowed;

    Call(const std::string& function, Node* arg, bool borrowed = false):
        function(function),
        arguments(1, arg),
        borrowed(borrowed) {}

    Call(const std::string& function, const std::vector<Node*>& args, bool borrowed = false):
        function(function),
        arguments(args),
        borrowed(borrowed) {}

    ~Call(void) {
        if (not borrowed) {
            for (auto arg : arguments) delete arg;
        }
    }
};

}
}
