#pragma once

#include <string>

#include <stencila/syntax-tree.hpp>

namespace Stencila {
namespace Syntax {

/**
 * Base class for language parsers
 */
class Parser {
 public:

    /**
     * Destructor
     */
    ~Parser(void);

    /**
     * Get syntax tree root created during parsing
     */
    const Node* root(void) const;

    /**
     * Set syntax tree root during parsing
     */
    void root(Node* node);

 protected:
    /**
     * Root node of the syntax tree that
     * is created during parsing
     */
    Node* root_ = nullptr;
};

}
}
