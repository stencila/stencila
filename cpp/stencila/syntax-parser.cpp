#include <cstdlib>
#include <string>
#include <vector>

#include <stencila/syntax-parser.hpp>

namespace Stencila {
namespace Syntax {

const Node* Parser::root(void) const {
    return root_;
}

void Parser::root(Node* root) {
    root_ = root;
}

}
}
