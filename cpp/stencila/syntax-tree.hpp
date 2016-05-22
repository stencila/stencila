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

    explicit Boolean(const std::string& string) {
        std::string v = string;
        boost::algorithm::to_lower(v);
        if (v == "true") {
            value = true;
        } else if (v == "false") {
            value = false;
        } else {
            STENCILA_THROW(Exception, "Invalid string value for Boolean.\n  value: " + string);
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
    std::string symbol;
    Node* left;
    Node* right;

    Binary(const std::string& symbol, Node* left, Node* right):
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

    Call(const std::string& function, Node* arg):
        function(function),
        arguments(1, arg) {}

    Call(const std::string& function, const std::vector<Node*>& args):
        function(function),
        arguments(args) {}

    ~Call(void) {
        for (auto arg : arguments) delete arg;
    }
};

}
}
