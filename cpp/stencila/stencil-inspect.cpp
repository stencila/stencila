#include <vector>
#include <algorithm>

#include <boost/regex.hpp>

#include <stencila/stencil.hpp>

namespace Stencila {

const std::vector<std::string> Stencil::tags = {
    "section","nav","article","aside","address","h1","h2","h3","h4","h5","h6","p","hr","pre","blockquote","ol","ul","li","dl","dt","dd",
    "figure","figcaption","div","a","em","strong","small","s","cite","q","dfn","abbr","data","time","code","var","samp","kbd","sub","sup","i","b","u","mark",
    "rt","rp","bdi","bdo","span","br","wbr","ins","del","table","caption","colgroup","col","tbody","thead","tfoot","tr","td","th"
};

const std::vector<std::string> Stencil::directives = {
    "data-code","data-format","data-size",
    "data-text","data-with",
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

Stencil::Code Stencil::parse_code(const std::string& attribute){
    static const boost::regex pattern("^(\\w+(\\s*,\\s*\\w+)*)(\\s+\\w+)?(\\s+([0-9]*\\.?[0-9]+)x([0-9]*\\.?[0-9]+)(\\s*\\w+)?)?$");
    boost::smatch match;
    if(boost::regex_search(attribute, match, pattern)){
        Code directive;
        // Comma separated list of compatible contexts
        auto contexts = split(match[1].str(),",");
        for(auto& context : contexts) trim(context);
        for(const auto& context : contexts){
            if(not(
                context=="py" or
                context=="r"
            )) STENCILA_THROW(Exception,"Context type <"+context+"> is not valid");
        }
        directive.contexts = contexts;
        // Format
        auto format = match[3].str();
        trim(format);
        if(format.length() and not(
            format=="text" or 
            format=="png" or format=="jpg" or format=="svg"
        )) STENCILA_THROW(Exception,"Format <"+format+"> is not valid");
        directive.format = format;
        // Size
        directive.width = match[5].str();
        directive.height = match[6].str();
        auto units = match[7].str();
        trim(units);
        if(units.length() and not(
            units=="cm" or units=="in" or units=="px"
        )) STENCILA_THROW(Exception,"Size units <"+units+"> is not valid");
        directive.units = units;
        return directive;
    } else {
        STENCILA_THROW(Exception,"Syntax error in code directive attribute <"+attribute+">");
    }
}

Stencil::For Stencil::parse_for(const std::string& attribute){
    static const boost::regex pattern("^(\\w+)\\s+in\\s+(.+)$");
    boost::smatch match;
    if(boost::regex_search(attribute, match, pattern)) {
        For directive;
        directive.name = match[1].str();
        directive.expr = match[2].str();
        return directive;
    }
    else {
        STENCILA_THROW(Exception,"Syntax error in for directive attribute <"+attribute+">");
    }
}

Stencil& Stencil::sanitize(void) {
    return *this;
};

}
