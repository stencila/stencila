#include <boost/tokenizer.hpp>

#include <stencila/sheet.hpp>

namespace Stencila {

Sheet& Sheet::load_separated(std::istream& stream, const std::string& format, const std::string& at) {
    char separator = ',';
    if (format == "tsv") {
        separator = '\t';
    } else if (format == "csv") {
        separator = ',';
    }

    boost::escaped_list_separator<char> escaped_list_separator(
        '\\',
        separator,
        '\"'
    );
    std::vector<Cell> cells;
    auto indices = index(at);
    auto row = indices[0];
    auto col_start = indices[1];
    std::string line;
    while (std::getline(stream, line)) {
        boost::tokenizer<decltype(escaped_list_separator)> tokenizer(
            line,
            escaped_list_separator
        );
        auto col = col_start;
        for (const auto& value : tokenizer) {
            Cell cell;
            cell.id = identify(row, col++);
            cell.source(value);
            cells.push_back(cell);
        }
        row++;
    }

    update(cells);

    return *this;
}

}
