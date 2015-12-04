//http://www.boost.org/doc/libs/1_59_0/libs/graph/doc/
//https://en.wikipedia.org/wiki/Topological_sorting

#include <boost/algorithm/string.hpp>
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

Sheet& Sheet::read(std::istream& stream){
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

Sheet& Sheet::read(const std::string& path) {
    std::ifstream file(path);
    return read(file);
}

Sheet& Sheet::write(std::ostream& stream) {
    return *this;
}

Sheet& Sheet::write(const std::string& path) {
    std::ofstream file(path);
    return write(file);
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
