#pragma once

#include <vector>
#include <string>

#include <Rcpp.h>
// Undefine some macros that R defines which clash
// with those used below
#undef ERROR

#include <stencila/spread.hpp>
#include <stencila/string.hpp>

#include "stencila.hpp"

namespace Stencila {

class RSpread : public Spread {
 public:
    explicit RSpread(SEXP sexp) {
        spread_ = Rcpp::Environment(sexp);
    }

    virtual ~RSpread(void) {
    }

    /**
     * @name Spread interface implementation
     *
     * For documentation on these methods see the base abstract class `Spread`
     *
     * @{
     */

    std::string execute(const std::string& package) {
        return call_(".execute", package);
    }

    std::string evaluate(const std::string& expression) {
        return call_(".evaluate", expression, "eval", true);
    }

    std::string set(const std::string& id, const std::string& expression, const std::string& name = "") {
        return call_(".set", id, expression, name);
    }

    std::string get(const std::string& name) {
        return call_(".get", name);
    }

    std::string clear(const std::string& id = "", const std::string& name = "") {
        return call_(".clear", id, name);
    }

    std::string list(void) {
        return call_(".list");
    }

    std::string collect(const std::vector<std::string>& cells) {
        return "c(" + join(cells, ",") + ")";
    }

    std::string depends(const std::string& expression) {
        return call_(".depends", expression);
    }

    std::vector<std::string> functions(void) {
        return split(call_(".functions"), ",");
    }

    Function function(const std::string& name) {
        Rcpp::Function method = spread_.get(".function");
        Rcpp::Language call(method, name);
        Function func = from<Function>(call.eval());
        return func;
    }

    void read(const std::string& path) {
        call_(".read", path);
    }

    void write(const std::string& path) {
        call_(".write", path);
    }

    /**
     * @}
     */

    /**
     * Initialisation of this class
     */
    static void class_init(void) {
        Class::set(RSpreadType, {
            "RSpread"
        });
    }

 private:
    /**
     * An Rcpp object which represents this spread on the R side
     */
    Rcpp::Environment spread_;

    /**
     * Call a method on the R side spread
     * 
     * Currently, this function only handles strings returned from R and then casts those
     * using boost::lexical_cast. I got serious errors of the form:
     *    memory access violation at address: 0x7fff712beff8: no mapping at fault address
     * when trying to use Rcpp::as<bool> or Rcpp::as<int> even when checking the returned SEXP was
     * the correct type
     */
    template<
        typename Result = std::string,
        typename... Args
    >
    Result call_(const char* name, Args... args) {
        Rcpp::Function func = spread_.get(name);
        Rcpp::Language call(func, args...);
        SEXP result = call.eval();
        if (TYPEOF(result) != STRSXP) STENCILA_THROW(Exception,"R-side methods should return a string");
        return unstring<Result>(Rcpp::as<std::string>(result));
    }
};

}
