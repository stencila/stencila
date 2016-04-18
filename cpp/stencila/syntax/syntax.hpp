#pragma once

#include <iostream>

#include <boost/algorithm/string.hpp>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Syntax {

struct Node {
    enum Type {
        boolean, number, string,
        identifier, range,
        binary,
        call
    } type;

    Node(Type type):
        type(type) {}

    virtual ~Node(void) {
    }
};

struct Boolean : Node {
    bool value;

    Boolean(const char* string):
        Node(number) {
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

    Number(const char* value):
        Node(number),
        value(value) {}

    ~Number(void){
    }
};

struct String : Node {
    std::string value;

    String(const char* value):
        Node(string),
        value(value) {}

    ~String(void){
    }
};

struct Identifier : Node {
    std::string value;

    Identifier(const char* value):
        Node(identifier),
        value(value) {}

    ~Identifier(void){
    }
};

struct Range : Node {
    Node* first;
    Node* last;

    Range(Node* first, Node* last):
        Node(range), first(first), last(last) {}

    ~Range(void){
    }
};

struct Binary : Node {
    char symbol;
    Node* left;
    Node* right;

    Binary(char symbol, Node* left, Node* right):
        Node(binary),
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
        Node(call),
        function(function),
        arguments(1, arg),
        borrowed(borrowed) {}

    Call(const std::string& function, const std::vector<Node*>& args, bool borrowed = false):
        Node(call),
        function(function),
        arguments(args),
        borrowed(borrowed) {}

    ~Call(void) {
        if (not borrowed) {
            for (auto arg : arguments) delete arg;
        }
    }
};


class Parser {
 public:
    std::string message;
    Node* root = nullptr;

    void parse(const std::string& line);

    void show(void) const;
};


class Generator {
 public:
    /**
     * Visit a node
     */
    void visit(const Node* node) {
        if (auto deriv = dynamic_cast<const Number*>(node)) {
            visit_number(deriv);
        } else if (auto deriv = dynamic_cast<const Identifier*>(node)) {
            visit_identifier(deriv);
        } else if (auto deriv = dynamic_cast<const Range*>(node)) {
            visit_range(deriv);
        } else if (auto deriv = dynamic_cast<const Binary*>(node)) {
            visit_binary(deriv);
        } else if (auto deriv = dynamic_cast<const Call*>(node)) {
            visit_call(deriv);
        }
    }

    virtual void visit_number(const Number*) {}

    virtual void visit_identifier(const Identifier*) {}

    virtual void visit_range(const Range*) {}

    virtual void visit_binary(const Binary*) {}

    virtual void visit_call(const Call*) {}
};


class StreamGenerator : public Generator {
 public:
    StreamGenerator(std::ostream& stream):
        stream_(stream) {}

 protected:

    std::ostream& stream_;
};


class TreeGenerator : public StreamGenerator {
 public:
    TreeGenerator(std::ostream& stream):
        StreamGenerator(stream) {}


    void visit_number(const Number* number) {
        line() << "number " << number->value << "\n";
    }

    void visit_binary(const Binary* binary) {
        line() << "binary " << binary->symbol << "\n";
        indent();
        visit(binary->left);
        visit(binary->right);
        outdent();
    }

    void visit_call(const Call* call) {
        line() << "call " << call->function << "\n";
        indent();
        for(auto arg : call->arguments) visit(arg);
        outdent();
    }

    std::ostream& line(void) {
        stream_ << indent_;
    }

    void indent(void) {
        indent_.push_back('\t');
    }

    void outdent(void) {
        if (indent_.size() == 0) {
            STENCILA_THROW(Exception, "Outdent without mathcing indent!");
        }
        indent_.pop_back();
    }

 protected:

    std::string indent_;
};


/**
 * A syntax generator that will be useful as
 * a base class for language specific generators.
 * It outputs nodes in ways that is common to many
 * languages so that you don't need to implement a 
 * `visit(const Type*)` for evry type of node
 */
class CodeGenerator : public StreamGenerator {
 public:
    CodeGenerator(std::ostream& stream, const std::string source):
        StreamGenerator(stream), source_(source) {}

    virtual void visit_number(const Number* number) {
        stream_ << number->value;
    }

    virtual void visit_identifier(const Identifier* id) {
        stream_ << id->value;
    }

    virtual void visit_range(const Range* range) {
        visit(range->first);
        stream_ << ":";
        visit(range->last);
    }

    virtual void visit_binary(const Binary* binary) {
        visit(binary->left);
        stream_ << binary->symbol;
        visit(binary->right);
    }

    virtual void visit_call(const Call* call){
        // Translate call based on source language
        // The translation will often be a modified Call
        // node but may need to be another type of Node
        const Node* node;
        bool created = true;
        if (source_ == "excel") {
            node = translate_excel_call(call, &created);
        } else {
            node = call;
            created = false;
        }
        // Do actual code generation for the translated call
        if(auto call = dynamic_cast<const Call*>(node)){
            stream_ << call->function << "(";
            auto last = call->arguments.back();
            for (auto arg : call->arguments) {
                visit(arg);
                if (arg != last) stream_ << ",";
            }
            stream_ << ")";
        } else {
            visit(node);
        }
        // Cleanup
        if (created) delete node;
    }

    virtual const Node* translate_excel_call(const Call* call, bool* created) {
        // Default, do no translation
        *created = false;
        return call;
    }

 protected:
    std::string source_;
};


}
}
