#include <vector>
#include <algorithm>

#include <stencila/stencil.hpp>

namespace Stencila {

const std::vector<std::string> Stencil::tags = {
    "section","nav","article","aside","address","h1","h2","h3","h4","h5","h6","p","hr","pre","blockquote","ol","ul","li","dl","dt","dd",
    "figure","figcaption","div","a","em","strong","small","s","cite","q","dfn","abbr","data","time","code","var","samp","kbd","sub","sup","i","b","u","mark",
    "rt","rp","bdi","bdo","span","br","wbr","ins","del","table","caption","colgroup","col","tbody","thead","tfoot","tr","td","th"
};

bool Stencil::tag(const std::string& name){
    return std::find(tags.begin(),tags.end(),name)!=tags.end();
}

Stencil& Stencil::sanitize(void) {
    return *this;
};

}
