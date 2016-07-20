/**
 * Exposes the `Syntax` namespace into R so that people can use
 * parse and generator independently of sheets.
 */

#include <stencila/string.hpp>
#include <stencila/syntax-excel.hpp>
#include <stencila/syntax-r.hpp>
using namespace Stencila; 
using namespace Stencila::Syntax;

#include "extension.hpp"

namespace {

/**
 * Generator for converting an AST into a R object
 */
class RObjectGenerator : public Generator {
 public:

    SEXP generate(const Node* node, const std::string mode="l") {
        using Rcpp::_;

        if (auto deriv = dynamic_cast<const Boolean*>(node)) {
            if (mode == "l") {
                return Rcpp::List::create(
                    _["type"] = "Boolean",
                    _["value"] = deriv->value
                );
            } else {
                return Rcpp::LogicalVector::create(deriv->value);
            }
        } else if (auto deriv = dynamic_cast<const Number*>(node)) {
            if (mode == "l") {
                return Rcpp::List::create(
                    _["type"] = "Number",
                    _["value"] = deriv->value
                );
            } else {
                return Rcpp::NumericVector::create(unstring<double>(deriv->value));
            }
        } else if (auto deriv = dynamic_cast<const String*>(node)) {
            if (mode == "l") {
                return Rcpp::List::create(
                    _["type"] = "String",
                    _["value"] = deriv->value
                );
            } else {
                return Rcpp::CharacterVector::create(deriv->value);
            }
        } else if (auto deriv = dynamic_cast<const Identifier*>(node)) {
            if (mode == "l") {
                return Rcpp::List::create(
                    _["type"] = "Identifier",
                    _["value"] = deriv->value
                );
            } else {
                return Rcpp::Symbol(deriv->value);
            }
        } else if (auto deriv = dynamic_cast<const Range*>(node)) {
            auto list = Rcpp::List::create(
                _["first"] = generate(deriv->first, mode),
                _["last"] = generate(deriv->last, mode)
            );
            if (mode == "l") list["type"] = "Range";
            else list.attr("class") = "Range";
            return list;
        } else if (auto deriv = dynamic_cast<const Binary*>(node)) {
            auto list = Rcpp::List::create(
                _["left"] = generate(deriv->left, mode),
                _["right"] = generate(deriv->right, mode)
            );
            if (mode == "l") list["type"] = "Binary";
            else list.attr("class") = "Binary";
            return list;
        } else if (auto deriv = dynamic_cast<const Call*>(node)) {
            auto args = Rcpp::List(deriv->arguments.size());
            unsigned int index = 0;
            for (const Node* arg : deriv->arguments) {
                args[index++] = generate(arg, mode);
            }
            auto list = Rcpp::List::create(
                _["function"] = deriv->function,
                _["arguments"] = args
            );
            if (mode == "l") list["type"] = "Call";
            else list.attr("class") = "Call";
            return list;
        }
        return null;
    }
};

}

/**
 * Convert an Excel formula into an AST
 */
STENCILA_R_FUNC excel_ast(SEXP excel, SEXP mode){
    STENCILA_R_BEGIN
        return RObjectGenerator().generate(
            ExcelParser().parse(as<std::string>(excel)),
            as<std::string>(mode)
        );
    STENCILA_R_END
}

/**
 * Convert an Excel formula into a R expression
 * (i.e. using compatability functions like SUM)
 */
STENCILA_R_FUNC excel_r(SEXP excel){
    STENCILA_R_BEGIN
        return wrap(
            ExcelToRSheetGenerator().generate(
                ExcelParser().parse(as<std::string>(excel))
            )
        );
    STENCILA_R_END
}
