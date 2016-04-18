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

    template<typename Arg>
    void out(Arg arg) {
        stream_<<arg;
    }

    template<typename Arg,typename... Args>
    void out(Arg arg, Args... args) {
        out(arg);
        out(args...);
    }

 protected:
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


    using Generator::visit;
    
    void visit(const Node* node, const std::string& source) {
        source_ = source;
        visit(node);
    }

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

    virtual void visit_call(const Call* call) {
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
        if (auto call = dynamic_cast<const Call*>(node)) {
            out(call->function, "(");
            auto last = call->arguments.back();
            for (auto arg : call->arguments) {
                visit(arg);
                if (arg != last) out(",");
            }
            out(")");
        } else {
            visit(node);
        }
        // Cleanup
        if (created) delete node;
    }

    /**
     * Translate a call to an excel function into the target language
     * 
     * @param  call    Call node to translate
     * @param  created Flag to indicate if a new node was created (and thus needs to be deleted)
     */
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
