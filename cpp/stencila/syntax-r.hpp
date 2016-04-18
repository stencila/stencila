#pragma once

#include <map>
#include <string>

#include <stencila/syntax-generator.hpp>

namespace Stencila {
namespace Syntax {

class ExcelToRGenerator : public CodeGenerator {
 public:
    using CodeGenerator::CodeGenerator;

    /**
     * Translate an Excel call to an R call
     */
    void visit_call(const Call* call);

    /**
     * Map for translating Excel function names to R function names
     */
    static const std::map<std::string, std::string> function_map;

};

}
}
