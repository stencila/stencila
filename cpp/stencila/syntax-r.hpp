#pragma once

#include <map>
#include <string>

#include <stencila/syntax-generator.hpp>

namespace Stencila {
namespace Syntax {

/**
 * This R code generator does not attempt to convert Excel function
 * names and instead relies on the existence of compatability functions with the
 * same names in R e.g. `AVERAGE`
 */
class ExcelToRSheetGenerator : public CodeGenerator {
 public:
    using CodeGenerator::CodeGenerator;

    void visit_call(const Call* call);
};

/**
 * This R code generator converts Excel function calls into their
 * equivalents in R using only R functions. As such it does not rely on the
 * compatability functions available in the stencila R package.
 */
class ExcelToRGenerator : public CodeGenerator {
 public:
    using CodeGenerator::CodeGenerator;

    void visit_call(const Call* call);
};

}
}
