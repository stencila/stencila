#include <cstdlib>
#include <string>
#include <vector>

#include <stencila/syntax-parser.hpp>

namespace Stencila {
namespace Syntax {

Parser::~Parser(void) {
	delete root_;
}

const Node* Parser::root(void) const {
    return root_;
}

void Parser::root(Node* root) {
    delete root_;
    root_ = root;
}

}
}
