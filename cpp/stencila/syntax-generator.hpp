#pragma once

#include <iostream>
#include <sstream>
#include <string>
#include <memory>

#include <stencila/syntax-tree.hpp>

namespace Stencila {
namespace Syntax {

/**
 * Base class for syntax generators
 */
class Generator {
 public:

    /**
     * Visit a node of a syntax tree
     *
     * This method dispatches to the `visit_xxx` method of derived classes
     */
    void visit(const Node* node) {
        if (auto deriv = dynamic_cast<const Boolean*>(node)) {
            visit_boolean(deriv);
        } else if (auto deriv = dynamic_cast<const Number*>(node)) {
            visit_number(deriv);
        } else if (auto deriv = dynamic_cast<const String*>(node)) {
            visit_string(deriv);
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

    /**
     * @name Visit methods for node types
     *
     * Derived classes should override these
     *
     * @{
     */
    virtual void visit_boolean(const Boolean*) {}

    virtual void visit_number(const Number*) {}

    virtual void visit_string(const String*) {}

    virtual void visit_identifier(const Identifier*) {}

    virtual void visit_range(const Range*) {}

    virtual void visit_binary(const Binary*) {}

    virtual void visit_call(const Call*) {}

    /**
     * @}
     */
};


/**
 * A base class for generators that output to stream
 *
 * This is likely to including most generators
 */
class StreamGenerator : public Generator {
 public:

    StreamGenerator(std::ostream& stream):
        stream_(stream) {}

    StreamGenerator(void):
        stringstream_(new std::ostringstream),
        stream_(*stringstream_) {}

    ~StreamGenerator(void){
        delete stringstream_;
    }

    template<typename Arg>
    void out(Arg arg) {
        stream_<<arg;
    }

    template<typename Arg,typename... Args>
    void out(Arg arg, Args... args) {
        out(arg);
        out(args...);
    }

    std::string generate(const Node* node) {
        if (stringstream_) {
            stringstream_->str("");
            stringstream_->clear();
            visit(node);
            return stringstream_->str();
        } else {
            STENCILA_THROW(Exception, "StreamGenerator initialised with an external stream");
        }
    }

 protected:
    std::ostringstream* stringstream_ = nullptr;
    std::ostream& stream_;
};


/**
 * A generator that produces a text representation
 * of a syntax tree which can be useful for debugging
 */
class TreeGenerator : public StreamGenerator {
 public:

    using StreamGenerator::StreamGenerator;

    void visit_boolean(const Boolean* boolean) {
        line("boolean ",boolean->value);
    }

    void visit_number(const Number* number) {
        line("number ", number->value);
    }

    void visit_string(const String* string) {
        line("string ", string->value);
    }

    void visit_identifier(const Identifier* node) {
        line("identifier ", node->value);
    }

    void visit_range(const Range* node) {
        line("range ", node->first);
        indent();
        visit(node->first);
        visit(node->last);
        outdent();
    }

    void visit_binary(const Binary* binary) {
        line("binary ", binary->symbol);
        indent();
        visit(binary->left);
        visit(binary->right);
        outdent();
    }

    void visit_call(const Call* call) {
        line("call ", call->function);
        indent();
        for (auto arg : call->arguments) visit(arg);
        outdent();
    }

    template<typename... Args>
    void line(Args... args) {
        stream_ << indent_;
        out(args...);
        stream_ << "\n";
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
 * `visit_xxx` for evry type of node type.
 */
class CodeGenerator : public StreamGenerator {
 public:

    using StreamGenerator::StreamGenerator;

    virtual void visit_boolean(const Boolean* node) {
        out(node->value);
    }

    virtual void visit_number(const Number* node) {
        out(node->value);
    }

    virtual void visit_string(const String * node) {
        out(node->value);
    }

    virtual void visit_identifier(const Identifier* node) {
        out(node->value);
    }

    virtual void visit_range(const Range* node) {
        visit(node->first);
        out(":");
        visit(node->last);
    }

    virtual void visit_binary(const Binary* node) {
        visit(node->left);
        out(node->symbol);
        visit(node->right);
    }

    virtual void visit_call(const Call* node) {
        out(node->function, "(");
        visit_call_args(node->arguments);
        out(")");
    }

    virtual void visit_call_args(const std::vector<Node*>& arguments, const std::string& separator=",") {
        auto last = arguments.back();
        for (auto arg : arguments) {
            visit(arg);
            if (arg != last) out(separator);
        }
    }

};

}
}
