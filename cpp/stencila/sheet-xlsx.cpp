#include <iostream>

#include <zip.h>

#include <stencila/sheet.hpp>
#include <stencila/syntax-excel.hpp>
#include <stencila/syntax-r.hpp>

namespace Stencila {

Sheet& Sheet::load_xlsx(const std::string& path, const std::string& sheet, const std::string& at, bool execute) {
    // Open the zip .xlsx file
    int error = 0;
    zip* archive = zip_open(path.c_str(), ZIP_RDONLY, &error);
    if (error) {
        STENCILA_THROW(Exception, "Could not read zip file\n  path: " + path);
    }

    // Local function for getting a XML document
    auto xml = [&archive](const std::string& name) {
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
        return Xml::Document(content);
    };

    // Load shared strings
    std::vector<std::string> strings;
    for (auto string : xml("xl/sharedStrings.xml").filter("sst si t")) {
        strings.push_back(string.text());
    }

    // Load the worksheet data
    auto worksheet = xml("xl/worksheets/" + sheet + ".xml");
    auto data = worksheet.find("sheetData");


    // Formula translation function
    // TODO select correct generator
    Syntax::ExcelParser parser;
    Syntax::ExcelToRSheetGenerator generator;
    auto translate = [&](const std::string& formula){
        return formula;
        // TODO Right now, this is not doing any translation!
        //return generator.generate(parser.parse(formula));
    };

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
                source = "= " + translate(formula);
            } else {
                if (type == "s") {
                    int index = unstring<int>(value);
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
            cell.value = value;
            cells.push_back(cell);
        }
    }

    update(cells,execute);

    return *this;
}

}
