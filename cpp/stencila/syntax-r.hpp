#pragma once

#include <map>
#include <string>

#include <stencila/syntax/generator.hpp>

namespace Stencila {
namespace Syntax {

class RGenerator : public CodeGenerator {
 public:
    using CodeGenerator::CodeGenerator;

    /**
     * Translate an Excel call to an R call
     */
    virtual const Node* translate_excel_call(const Call* call, bool* created);

    /**
     * Map for translating Excel function names to R function names
     */
    static const std::map<std::string, std::string> excel_function_map;

};

}
}
