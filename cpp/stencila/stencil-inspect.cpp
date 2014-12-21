#include <vector>
#include <algorithm>

#include <stencila/stencil.hpp>

namespace Stencila {

const std::vector<std::string> Stencil::tags = {
    "section","nav","article","aside","address","h1","h2","h3","h4","h5","h6","p","hr","pre","blockquote","ol","ul","li","dl","dt","dd",
    "figure","figcaption","div","a","em","strong","small","s","cite","q","dfn","abbr","data","time","code","var","samp","kbd","sub","sup","i","b","u","mark",
    "rt","rp","bdi","bdo","span","br","wbr","ins","del","table","caption","colgroup","col","tbody","thead","tfoot","tr","td","th"
};

const std::vector<std::string> Stencil::directives = {
    "data-exec","data-format","data-size",
    "data-write",
    "data-refer",
    "data-with",
    "data-if","data-elif","data-else",
    "data-switch","data-case","data-default",
    "data-for",
    "data-include","data-version","data-select","data-set",
        "data-delete","data-replace","data-change","data-before","data-after","data-prepend","data-append",
    "data-macro","data-par",
};

const std::vector<std::string> Stencil::flags = {
    "data-const","data-hash","data-out",
    "data-off","data-index","data-lock","data-included",
    "data-error"
};

bool Stencil::tag(const std::string& name){
    return std::find(tags.begin(),tags.end(),name)!=tags.end();
}

bool Stencil::directive(const std::string& attr){
    return std::find(directives.begin(),directives.end(),attr)!=directives.end();
}

bool Stencil::flag(const std::string& attr){
    return std::find(flags.begin(),flags.end(),attr)!=flags.end();
}

Stencil& Stencil::sanitize(void) {
    return *this;
};

}
