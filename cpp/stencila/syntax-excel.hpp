#pragma once

#include <stencila/syntax-parser.hpp>

namespace Stencila {
namespace Syntax {

class ExcelParser : public Parser {
 public:
    const Node* parse(const std::string& string);
};

}
}
