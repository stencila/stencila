//http://www.boost.org/doc/libs/1_59_0/libs/graph/doc/
//https://en.wikipedia.org/wiki/Topological_sorting

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <iostream>

#include <stencila/sheet.hpp>
#include <stencila/exception.hpp>

namespace Stencila {

Sheet::Sheet(void){
}

Sheet::Sheet(const std::string& from){
    initialise(from);
}

Sheet::~Sheet(void){
}

Sheet& Sheet::initialise(const std::string& from){
    if(boost::filesystem::exists(from)){
        read(from);
    } else {
        std::string path = Component::locate(from);
        if(path.length()) read(path);
        else STENCILA_THROW(Exception,"No sheet found with path or address:\n path: "+from);
    }        
    return *this;
}

Sheet& Sheet::load(std::istream& stream, const std::string& format){
    unsigned int row = 0;
    std::string line;
    while(std::getline(stream,line)){
        std::vector<std::string> cells;
        boost::split(cells,line,boost::is_any_of("\t"));
        unsigned int col = 0;
        for(auto cell : cells){
            cells_[identify(row,col)] = {
                row,
                col,
                cell
            };
            col++;
        }
        row++;
    }
    return *this;
}

Sheet& Sheet::load(const std::string& string, const std::string& format) {
    std::istringstream stream(string);
    return load(stream,format);
}

Sheet& Sheet::dump(std::ostream& stream, const std::string& format) {
    // TODO
    return *this;
}

std::string Sheet::dump(const std::string& format) {
    // TODO
    return "";
}

Sheet& Sheet::import(const std::string& path){
    if(not boost::filesystem::exists(path)){
        STENCILA_THROW(Exception,"File not found\n path: "+path);
    }
    std::string ext = boost::filesystem::extension(path);
    if(ext==".tsv"){
        std::ifstream file(path);
        load(file,"tsv");
    }
    else STENCILA_THROW(Exception,"File extension not valid for a sheet\n extension: "+ext);
    return *this;
}

Sheet& Sheet::export_(const std::string& path){
    std::string ext = boost::filesystem::extension(path);
    if(ext==".tsv"){
        std::ofstream file(path);
        dump(file,"tsv");
    }
    else STENCILA_THROW(Exception,"File extension not valid for a sheet\n extension: "+ext);
    return *this;
}

Sheet& Sheet::read(const std::string& directory){
    Component::read(directory);
    import("sheet.tsv");
    return *this;
}

Sheet& Sheet::write(const std::string& directory){
    // TODO
    return *this;
}

Sheet& Sheet::compile(void){
    auto home = boost::filesystem::path(path(true));
    auto filepath = (home/"index.html").string();
    export_(filepath);
    return *this;
}

std::string Sheet::identify(unsigned int row, unsigned int col){
    std::string id;
    while(true){
        int mod = (col % 26) + 65;
        col /= 26;
        id = char(mod) + id;
        if(col > 0) col--;
        else if(col == 0) break;
    }
    id += string(row+1);
    return id;
}

std::array<std::string,3> Sheet::parse(const std::string& content){
    static const boost::regex regex("(.*?)( = (.+?))?( = (.+?))?");
    boost::smatch match;
    if(boost::regex_match(content,match,regex)){
        return {match.str(1),match.str(3),match.str(5)};
    }
    else {
        STENCILA_THROW(Exception,"Error parsing content: \n  content: "+content);
    }
}

}
