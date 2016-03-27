#include <iostream>

#include <zip.h>

#include <stencila/sheet.hpp>

namespace Stencila {

// Local helper functions - may be better as lambdas below
namespace {

Xml::Document get_xml(zip* archive, const std::string& name) {
    std::string content;
    for (int index = 0; index < zip_get_num_entries(archive, 0); index++) {
        struct zip_stat stats;
        if (zip_stat_index(archive, index, 0, &stats) == 0) {
            if (stats.name == name) {
                auto file = zip_fopen_index(archive, index, 0);
                if (not file) {
                    STENCILA_THROW(Exception, "Could not read file from xlsx\n  name: " + std::string(stats.name));
                }

                unsigned int size = 0;
                char buffer[100];
                while (size != stats.size) {
                    auto chars = zip_fread(file, buffer, 100);
                    if (chars < 0) {
                        STENCILA_THROW(Exception, "Error reading file from xlsx\n  name: " + std::string(stats.name));
                    }
                    content.append(buffer, chars);
                    size += chars;
                }

                zip_fclose(file);
                break;
            }
        }
    }
    return content;
}

std::string xl_to_r(const std::string& xl) {
    auto r = xl;
    replace_all(r, "SUM(", "sum(");
    replace_all(r, "AVERAGE(", "mean(");
    return r;
}

}

Sheet& Sheet::load_xlsx(const std::string& path, const std::string& sheet, const std::string& at) {
    // Open the zip .xlsx file
    int error = 0;
    zip* archive = zip_open(path.c_str(), ZIP_RDONLY, &error);
    if (error) {
        STENCILA_THROW(Exception, "Could not read zip file\n  path: " + path);
    }

    // Load shared strings
    auto strings_xml = get_xml(archive, "xl/sharedStrings.xml").find("sst");
    auto strings_count = unstring<unsigned int>(strings_xml.attr("count"));
    auto strings_elems = strings_xml.children();
    if (strings_count != strings_elems.size()) {
        STENCILA_THROW(Exception, "Incompatible count and elements\n  count: " + string(strings_count) + "\n  elements:" + string(strings_elems.size()));
    }
    std::vector<std::string> strings(strings_count);
    int index = 0;
    for (auto string : strings_elems) {
        strings[index++] = string.find("t").text();
    }

    // Load the worksheet data
    auto worksheet = get_xml(archive, "xl/worksheets/" + sheet + ".xml");
    auto data = worksheet.find("sheetData");

    std::cout << data.dump(true);

    // Iterate over rows and columns and create cells
    std::vector<Cell> cells;
    for (auto row : data.children()) {
        for (auto col : row.children()) {
            // Get the various cell attributes
            auto id = col.attr("r");
            auto type = col.attr("t");
            auto value = col.find("v").text();
            auto formula = col.find("f").text();

            // Cell source is either translated formula
            // or cell value
            std::string source;
            if (formula.length()) {
                source = "= " + xl_to_r(formula);
            } else {
                if (type == "s") {
                    auto index = unstring<unsigned int>(value);
                    if (index < 0 or index >= strings.size()) {
                        STENCILA_THROW(Exception, "Shared string index is bad\n  index: " + value + "\n  size:" + string(strings.size()));
                    }
                    source = strings[index];
                } else {
                    source = value;
                }
            }
            
            Cell cell;
            cell.id = id;
            cell.source(source);
            cells.push_back(cell);
        }
    }

    update(cells);

    return *this;
}

}
