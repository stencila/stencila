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
        // Many Excel functions are equivalent to the corresponding
        // lower case R functions. So, this is the fallback...
        boost::to_lower(name);
        out(name, "(");
        visit_call_args(argv);
        out(")");
    }
}

}   
}
