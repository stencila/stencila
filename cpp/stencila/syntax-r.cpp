#include <map>

#include <boost/algorithm/string.hpp>

#include <stencila/syntax-r.hpp>

namespace Stencila {
namespace Syntax {

void ExcelToRGenerator::visit_call(const Call* call) {
    auto name = call->function;
    auto argv = call->arguments;
    auto argc = call->arguments.size();
    // Calls that require more sophisticated translation
    if (name == "AVERAGE" or name == "AVG") {
        out("mean(");
        if (argc > 1) out("c(");
        visit_call_args(argv);
        if (argc > 1) out(")");
        out(")");
    } 
    else {
        // Simple translation of function names
        auto trans = function_map.find(name);
        if (trans != function_map.end()) {
            // Translation found, use it.
            name = trans->second;
        } else {
            // Some Excel functions are equivalent to the corresponding
            // lower case R functions. So, this is the fallback...
            boost::to_lower(name);
        }
        out(name, "(");
        visit_call_args(argv);
        out(")");
    }
}

/**
 * A function name translation map
 *
 * All mappings and comments are from http://www.burns-stat.com/spreadsheet-r-vector/
 * Some additional mappings are provided at http://www.rforexcelusers.com/r-functions-excel-formulas/
 *
 * TODO : Fix up these mapping
 */
const std::map<std::string, std::string> ExcelToRGenerator::function_map = {
    {"ABS", "abs"},
    {"ADDRESS", ""},  // perhaps assign but there is probably a better way
    {"AND", "all"},  // more literally would be the & and && R operators
    {"AVERAGEIF", ""},  // subscript before using mean
    {"BESSELI", "besselI"},
    {"BESSELJ", "besselJ"},
    {"BESSELK", "besselK"},
    {"BESSELY", "besselY"},
    {"BETADIST", "pbeta"},
    {"BETAINV", "qbeta"},
    {"BINOMDIST", "pbinom ordbinom"},  // pbinom when cumulative, dbinom when not
    {"CEILING", "ceiling"},
    {"CELL", ""},  // str is sort of the same idea
    {"CHIDIST", "pchisq"},  // CHIDIST(x, df) is pchisq(x, df, lower.tail=FALSE)
    {"CHIINV", "qchisq"},  // CHIINV(p, df) is qchisq(1-p, df)
    {"CHISQDIST", "pchisq ordchisq"},  // pchisq when cumulative, dchisq when not
    {"CHISQINV", "qchisq"},
    {"CHITEST", "chisq.test"},
    {"CHOOSE", "switch"},
    {"CLEAN", "gsub"},
    {"COLS", "ncol"},  // (Works)
    {"COLUMNS", "ncol"},  // (Excel, OpenOffice)
    {"COLUMN", "col"},  // or probably more likely : or seq
    {"COMBIN", "choose"},
    {"CONCATENATE", "paste"},
    {"CONFIDENCE", ""},  // CONFIDENCE(alpha, std, n) is -qnorm(alpha/2) * std / sqrt(n)
    {"CORREL", "cor"},
    {"COUNT", "length"},
    {"COUNTIF", ""},  // get length of a subscripted object
    {"COVAR", "cov"},
    {"CRITBINOM", "qbinom"},  // CRITBINOM(n, p, a) is qbinom(a, n, p)
    {"DELTA", "all.equal oridentical"},  // all.equal allows for slight differences, and note that it does not return a logical if there’s a pertinent difference — you can wrap it in isTRUE if you want
    {"DGET", ""},  // use subscripting in R
    {"ERF", ""},  // see the example in ?"Normal"
    {"ERFC", ""},  // see the example in ?"Normal"
    {"EXP", "exp"},
    {"EXPONDIST", "pexp or dexp"},  // pexp when cumulative, dexp when not
    {"FACT", "factorial"},
    {"FACTDOUBLE", "dfactorial"},  // dfactorial is in the phangorn package
    {"FDIST", "pf"},  // FDIST(x, df1, df2) is pf(x, df1, df2, lower.tail=FALSE)
    {"FIND", "regexpr"},
    {"FINV", "qf"},  // FINV(p, df1, df2) is qf(1-p, df1, df2)
    {"FISHER", "atanh"},
    {"FISHERINV", "tanh"},
    {"FIXED", "format orsprintf orformatC"},
    {"FLOOR", "floor"},
    {"FORECAST", ""},  // predict on an lm object
    {"FREQUENCY", ""},  // you probably want to use cut and/or table
    {"FTEST", "var.test"},
    {"GAMMADIST", "pgamma ordgamma"},  // GAMMADIST(x, a, b, TRUE) is pgamma(x, a, scale=b)GAMMADIST(x, a, b, FALSE) is dgamma(x, a, scale=b)
    {"GAMMAINV", "qgamma"},  // GAMMAINV(p, a, b) is qgamma(p, a, scale=b)
    {"GAMMALN", "lgamma"},
    {"GAUSS", ""},  // GAUSS(x) is pnorm(x) - 0.5
    {"GCD", "gcd"},  // gcd is in the schoolmath package (and others). For more than two numbers you can do: Reduce(gcd, numVector)
    {"GEOMEAN", ""},  // exp(mean(log(x)))
    {"GESTEP", ">="},  // GESTEP(x, y) is as.numeric(x >= y) but R often coerces automatically if needed
    {"HARMEAN", "harmonic.mean"},  // harmonic.mean is in the psych package
    {"HLOOKUP", ""},  // use subscripting in R
    {"HYPGEOMDIST", "dhyper"},  // HYPGEOMDIST(x, a, b, n) is dhyper(x, b, n-b, a)
    {"IF", "if or ifelse"},  // see Circle 3.2 of The R Inferno on if versus ifelse
    {"IFERROR", "try ortryCatch"},
    {"INDEX", "["},  // use subscripting in R
    {"INDIRECT", "get"},  // or possibly the eval-parse-text idiom, or (better) make changes that simplify the situation
    {"INT", "floor"},  // danger: not the same as as.integer for negative numbers
    {"INTERCEPT", ""},  // (usually) the first element of coef of an lm object
    {"ISLOGICAL", "is.logical"},
    {"ISNUMBER", "is.numeric"},
    {"ISTEXT", "is.character"},
    {"KURT", "kurtosis"},  // kurtosis is in the moments package
    {"LARGE", ""},  // you can use subscripting after sort
    {"LCM", "scm"},  // scm is in the schoolmath package. For more than two numbers you can do: Reduce(scm, numVector)
    {"LEFT", "substr"},
    {"LEN", "nchar"},  // (Excel, OpenOffice)
    {"LENGTH", "nchar"},  // (Works)
    {"LINEST", ""},  // use lm
    {"LN", "log"},  // danger: the default base in R for log is e
    {"LOG", "log"},  // danger: the default base in spreadsheets for log is 10
    {"LOG10", "log10"},
    {"LOGINV", "qlnorm"},
    {"LOGNORMDIST", "plnorm"},
    {"LOWER", "tolower"},
    {"MATCH", "match or which"},  // match only does exact matches. Given that MATCH demands a sorted set of values for type 1 or -1, then MATCH(x, vec, 1) issum(x <= vec) and MATCH(x, vec, -1) is sum(x >= vec)when vec is sorted as MATCH assumes.
    {"MAX", "max or pmax"},  // max returns one value, pmax returns a vector
    {"MDETERM", "det"},
    {"MEDIAN", "median"},
    {"MID", "substr"},
    {"MIN", "min or pmin"},  // min returns one value, pmin returns a vector
    {"MINVERSE", "solve"},
    {"MMULT", "%*%"},
    {"MOD", "%%"},
    {"MODE", ""},  // the table function does the hard part. A crude approximation to MODE(x) is as.numeric(names(which.max(table(x))))
    {"MUNIT", "diag"},  // diag is much more general
    {"N", "as.numeric"},  // the correspondence is for logicals, as.numeric is more general
    {"NEGBINOMDIST", "dnbinom"},
    {"NORMDIST, NORMSDIST", "pnorm or dnorm"},  // pnorm when cumulative is true, dnorm when false
    {"NORMINV, NORMSINV", "qnorm"},
    {"NOT", "!"},
    {"NOW", "date orSys.time"},
    {"OR", "any"},  // the or operators in R are | and ||
    {"PEARSON", "cor"},
    {"PERCENTILE", "quantile"},
    {"PERCENTRANK", ""},  // similar to ecdf but the argument is removed from the distribution in PERCENTRANK
    {"PERMUT", ""},  // function(n,k) {choose(n,k) * factorial(k)}
    {"PERMUTATIONA", ""},  // PERMUTATIONA(n, k) is n^k
    {"PHI", "dnorm"},
    {"POISSON", "ppois or dpois"},  // ppois if cumulative, dpois if not
    {"POWER", "^"},
    {"PROB", ""},  // you can use the Ecdf function in the Hmisc package (the probabilities in the spreadsheet are the weights in Ecdf), then you can get the difference of that on the two limits
    {"PRODUCT", "prod"},
    {"PROPER", ""},  // see example in ?toupper
    {"QUARTILE", ""},  // use quantile
    {"QUOTIENT", "%/%"},
    {"RAND", "runif"},  // see an introduction to random generation in R
    {"RANDBETWEEN", ""},  // use sample
    {"RANK", "rank"},  // RANK has the "min" tie.method and defaults to biggest first. rank only has smallest first. To get biggest first in R you can do:length(x) + 1 - rank(x)
    {"REPLACE", "sub or gsub"},
    {"REPT", ""},  // use rep and paste or paste0
    {"RIGHT", "substring"},  // you’ll also need nchar to count the characters. Alternatively you can use str_sub in the stringr package with negative limits
    {"ROUND", "round"},  // note: round rounds exact halves to even (which avoids bias)
    {"ROUNDDOWN", "trunc"},  // trunc only goes to integers
    {"ROW", "row"},  // or probably more likely : or seq
    {"ROWS", "nrow"},
    {"RSQ", ""},  // in summary of an lm object
    {"SEARCH", "regexpr"},  // also see grep
    {"SIGN", "sign"},
    {"SKEW", "skewness"},  // skewness is in the moments package
    {"SLOPE", ""},  // in coef of an lm object
    {"SMALL", ""},  // you can use subscripting after sort
    {"SQRT", "sqrt"},
    {"STANDARDIZE", "scale"},
    {"STD", "sd"},  // (Works)
    {"STDEV", "sd"},  // (Excel, OpenOffice)
    {"STEYX", ""},  // predict on an lm object
    {"STRING", "format orsprintf orformatC orprettyNum"},  // (Works)
    {"SUBSTITUTE", "sub or gsub"},  // or possibly paste
    {"SUM", "sum"},  // sum is one of the few R functions that allow multiple data arguments
    {"SUMIF", ""},  // subscript before using sum
    {"SUMPRODUCT", "crossprod"},
    {"TDIST", "pt"},  // TDIST(abs(x), df, tails) is pt(-abs(x), df) * tails
    {"TEXT", "format orsprintf orformatC orprettyNum"},
    {"TINV", ""},  // TINV(x, df) is abs(qt(x/2, df))
    {"TODAY", "Sys.Date"},
    {"TRANSPOSE", "t"},
    {"TREND", ""},  // fitted of an lm object
    {"TRIM", "sub"},
    {"TRIMMEAN", "mean"},  // TRIMMEAN(x, tr) is mean(x, trim=tr/2)
    {"TRUNC", "trunc"},
    {"TTEST", "t.test"},
    {"TYPE", ""},  // similar concepts in R are typeof, mode, class. Use str to understand the structure of objects
    {"UPPER", "toupper"},
    {"VALUE", "as.numeric"},
    {"VAR", "var"},
    {"VLOOKUP", ""},  // use subscripting in R
    {"WEEKDAY", "weekdays"},
    {"WEIBULL", "pweibull ordweibull"},  // pweibull when cumulative, dweibull when not
    {"ZTEST", ""},  // use pnorm on the calculated statistic
};

}   
}
