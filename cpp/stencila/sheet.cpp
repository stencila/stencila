#include <vector>
#include <string>
#include <algorithm>

#include <boost/algorithm/string.hpp>
#include <boost/xpressive/xpressive.hpp>
#include <boost/filesystem.hpp>
#include <boost/graph/graphviz.hpp>
#include <boost/graph/topological_sort.hpp>
#include <boost/regex.hpp>

#include <stencila/sheet.hpp>
#include <stencila/component-page.hpp>
#include <stencila/exception.hpp>
#include <stencila/helpers.hpp>

namespace Stencila {

Sheet::Sheet(void) {
}

Sheet::Sheet(const std::string& from) {
    initialise(from);
}

Sheet::~Sheet(void) {
}

Component::Type Sheet::type(void) {
    return SheetType;
}

std::string Sheet::meta(const std::string& what) const {
    for (auto iter : cells_) {
        Cell& cell = iter.second;
        if (cell.name == what) {
            return cell.value;
        }
    }
    return "";
}

std::string Sheet::title(void) const {
    return meta("title");
}

std::string Sheet::description(void) const {
    return meta("description");
}

std::vector<std::string> Sheet::keywords(void) const {
    auto content = meta("keywords");
    if (content.length()) {
        auto values = split(content, ",");
        for (auto& value : values) trim(value);
        return values;
    } else {
        return {};
    }
}

std::vector<std::string> Sheet::authors(void) const {
    auto content = meta("authors");
    if (content.length()) {
        auto values = split(content, ",");
        for (auto& value : values) trim(value);
        return values;
    } else {
        return {};
    }
}

std::string Sheet::theme(void) const {
    return "";
}


Sheet& Sheet::initialise(const std::string& from) {
    if (boost::filesystem::exists(from)) {
        read(from);
    } else {
        std::string path = Component::locate(from);
        if (path.length()) read(path);
        else STENCILA_THROW(Exception, "No sheet found with path or address:\n path: "+from);
    }
    return *this;
}

Html::Fragment Sheet::html_table(unsigned int rows, unsigned int cols) const {
    if(rows==0 or cols==0){
        // Generate some sensible defaults
        auto extents = extent();
        if (rows == 0) rows = std::max(std::min(extents[0]+11, 200u), 50u);
        if (cols == 0) cols = std::max(std::min(extents[1]+11, 100u), 26u);
    }

    Html::Fragment frag("<table></table>");
    auto table = frag.select("table");
    auto tr = table.append("thead").append("tr");
    tr.append("th");
    for (unsigned int col = 0; col < cols; col++) {
        tr.append("th").text(identify_col(col));
    }
    auto tbody = table.append("tbody");
    for (unsigned int row = 0; row < rows; row++) {
        auto tr = tbody.append("tr");
        tr.append("th").text(identify_row(row));
        for (unsigned int col = 0; col < cols; col++) {
            auto td = tr.append("td");
            auto id = identify(row, col);
            const auto& iter = cells_.find(id);
            if (iter != cells_.end()) {
                const auto& cell = iter->second;
                if (cell.kind != Cell::blank_) {
                    td.attr("data-kind", cell.kind_string());
                    if (cell.name.length()) td.attr("data-name", cell.name);
                    if (cell.expression.length()) td.attr("data-expr", cell.expression);
                    if (cell.type.length()) td.attr("data-type", cell.type);
                    td.text(cell.value);
                }
            }
        }
    }
    return frag;
}

Sheet& Sheet::load(std::istream& stream, const std::string& format) {
    if (format == "tsv") {
        clear();
        unsigned int row = 0;
        std::string line;
        while (std::getline(stream, line)) {
            std::vector<std::string> cells;
            boost::split(cells, line, boost::is_any_of("\t"));
            unsigned int col = 0;
            for (auto cell : cells) {
                if (cell.length()) {
                    source(identify(row, col), cell);
                }
                col++;
            }
            row++;
        }
    }
    else STENCILA_THROW(Exception, "File extension not valid for loading a sheet\n extension: "+format);
    return *this;
}

Sheet& Sheet::load(const std::string& string, const std::string& format) {
    std::istringstream stream(string);
    return load(stream, format);
}

Sheet& Sheet::dump_script(std::ostream& stream, std::string assign, std::string terminate) {
    // Ensure `order_` and `cell.kind` are inititialised
    if(not prepared_) update();
    // Requirement cells first...
    bool requirements = false;
    for (const auto& iter : cells_) {
        const auto& cell = iter.second;
        if (cell.kind==Cell::requirement_ and cell.expression.length()) {
            stream  << translate(cell.expression)
                    << terminate;
            requirements = true;
        }
    }
    if (requirements) {
        stream << "\n";
    }
    // ...then other cells
    for (const auto& id : order_) {
        const auto& cell = cells_.at(id);
        if (not cell.kind==Cell::requirement_ and cell.expression.length()) {
            stream  << (cell.name.length()?cell.name:id)
                    << assign
                    << translate(cell.expression)
                    << terminate;
        }
    }
    return *this;
}

Sheet& Sheet::dump(std::ostream& stream, const std::string& format) {
    if (format == "tsv") {
        auto extents = extent();
        auto rows = extents[0] + 1;
        auto cols = extents[1] + 1;
        for (unsigned int row = 0; row < rows; row++) {
            std::vector<std::string> cells(cols);
            unsigned int col_max = 0;
            for (unsigned int col = 0; col < cols; col++) {
                auto src = source(identify(row, col));
                cells[col] = src;
                if (src.length()) {
                    col_max = col;
                }
            }
            for (unsigned int col = 0; col <= col_max; col++) {
                stream <<  cells[col];
                if (col < col_max) stream << "\t";
            }
            stream << "\n";
        }
    } else if (format == "r" or format=="py") {
        std::string assign = " = ";
        std::string termimate = "\n";
        if (format == "r") {
            assign = " <- ";
        }
        dump_script(stream, assign, termimate);
    }
    else STENCILA_THROW(Exception, "Format not valid for dumping a sheet\n format: "+format);
    return *this;
}

std::string Sheet::dump(const std::string& format) {
    std::ostringstream stream;
    dump(stream, format);
    return stream.str();
}

Sheet& Sheet::import(const std::string& path) {
    if (not boost::filesystem::exists(path)) {
        STENCILA_THROW(Exception, "File not found\n path: "+path);
    }
    std::string ext = boost::filesystem::extension(path);
    if (ext == ".tsv") {
        std::ifstream file(path);
        load(file, "tsv");
    }
    else STENCILA_THROW(Exception, "File extension not valid for a sheet\n extension: "+ext);
    return *this;
}

Sheet& Sheet::export_(const std::string& path) {
    std::string ext = boost::filesystem::extension(path);
    if (ext == ".tsv" or ext == ".r" or ext == ".py") {
        std::ofstream file(path);
        auto format = ext.substr(1);
        dump(file, format);
    }
    else STENCILA_THROW(Exception, "File extension not valid for a sheet\n extension: "+ext);
    return *this;
}

Sheet& Sheet::read(const std::string& directory) {
    using namespace boost::filesystem;
    // Call base method to set component path
    Component::read(directory);
    // Local function for reading a TSV mostly as per
    // http://dataprotocols.org/linear-tsv/
    auto read = [](const std::string& path) {
        std::vector<std::vector<std::string>> cells;
        std::ifstream file(path);
        std::string line;
        while (std::getline(file, line)) {
            std::vector<std::string> row;
            boost::split(row, line, std::bind1st(std::equal_to<char>(), '\t'));
            for(auto& value : row){
                boost::replace_all(value, "\\\\", "\\");  // Must come first
                boost::replace_all(value, "\\t", "\t");
                boost::replace_all(value, "\\n", "\n");
                boost::replace_all(value, "\\r", "\r");
            }
            cells.push_back(row);
        }
        return cells;
    };
    // Read cell source, types and values files
    auto dir = path() + "/"; 
    auto sources = read(dir + "sheet.tsv");
    auto types = read(dir + "sheet-types.tsv");
    auto values = read(dir + "sheet-values.tsv");
    // Local function for checking that types and values have a row/col
    // and return an empty string if they don't
    auto check = [](const std::vector<std::vector<std::string>>& cells, unsigned int row, unsigned int col) {
        if (cells.size() > row) {
            auto values = cells[row];
            if (values.size() > col) {
                return values[col];
            }
        }
        return std::string();
    };
    // Set stuff
    clear();
    unsigned int row = 0;
    for (const auto& strings : sources) {
        unsigned int col = 0;
        for (const auto& string : strings) {
            // Ignore empty cells
            if (string.length()) {
                auto id = identify(row, col);
                Cell& cell = cells_[id];
                source(id, string);
                cell.type = check(types, row, col);
                cell.value = check(values, row, col);
            }
            col++;
        }
        row++;
    }
    return *this;
}

Sheet& Sheet::write(const std::string& directory) {
    // Call base method to set component pth
    Component::write(directory);
    // Local function for escaping output mostly as per
    // http://dataprotocols.org/linear-tsv/
    auto escape = [](const std::string& value) {
        std::string copy = value;
        boost::replace_all(copy, "\t", "\\t");
        boost::replace_all(copy, "\n", "\\n");
        boost::replace_all(copy, "\r", "\\r");
        boost::replace_all(copy, "\\", "\\\\");
        return copy;
    };
    // Write cell source, types and values files
    auto dir = path() + "/";
    std::ofstream sources(dir + "sheet.tsv");
    std::ofstream types(dir + "sheet-types.tsv");
    std::ofstream values(dir + "sheet-values.tsv");
    auto extents = extent();
    auto rows = extents[0] + 1;
    auto cols = extents[1] + 1;
    for (unsigned int row = 0; row < rows; row++) {
        std::vector<std::array<std::string, 3>> cells(cols);
        unsigned int col_max = 0;
        for (unsigned int col = 0; col < cols; col++) {
            auto id = identify(row, col);
            auto src = source(id);
            const Cell& cell = cells_[id];
            cells[col] = {
                escape(src),
                escape(cell.type),
                escape(cell.value)
            };
            if (src.length()) {
                col_max = col;
            }
        }
        for (unsigned int col = 0; col <= col_max; col++) {
            sources <<  cells[col][0];
            types <<  cells[col][1];
            values <<  cells[col][2];
            if (col < col_max) {
                sources << "\t";
                types << "\t";
                values << "\t";
            }
        }
        sources << "\n";
        types << "\n";
        values << "\n";
    }

    return *this;
}

std::string Sheet::page(const Component* component) {
    return static_cast<const Sheet&>(*component).page();
}

std::string Sheet::page(void) const {
    // Get base document
    Html::Document doc = Component_page_doc<Sheet>(*this);
    Html::Node head = doc.find("head");
    Html::Node body = doc.find("body");

    // Add sheet to main#content
    auto main = body.select("main");
    main.attr("id", "content");
    main.append(html_table());

    return doc.dump(false);
}

Sheet& Sheet::page(const std::string& filename) {
    write_to(filename, page());
    return *this;
}

Sheet& Sheet::compile(void) {
    auto home = boost::filesystem::path(path(true));
    auto filepath = (home/"index.html").string();
    std::ofstream file(filepath);
    file << page();
    return *this;
}

std::string Sheet::serve(void) {
    return Component::serve(SheetType);
}

Sheet& Sheet::view(void) {
    Component::view(SheetType);
    return *this;
}

std::string Sheet::request(Component* component, const std::string& verb, const std::string& method, const std::string& body) {
    return static_cast<Sheet&>(*component).request(verb, method, body);
}

std::string Sheet::request(const std::string& verb, const std::string& method, const std::string& body) {
    Json::Document request;
    if (body.length()) {
        request.load(body);
    }

    // TODO: return error codes and messages instead of throwing exceptions

    // TODO : should be a GET but don't currently deal with query strings for parameters
    if (verb == "PUT" and method == "cell") {
        Cell cell;
        auto id = request["id"].as<std::string>();
        if (id.length()) {
            if (not is_id(id)) {
                STENCILA_THROW(Exception, "Not a valid id"); 
            }
            else {
                auto iter = cells_.find(id);
                if (iter != cells_.end()) {
                    cell = iter->second;
                } else {
                    STENCILA_THROW(Exception, "Not found");
                }
            }
        }
        else {
            auto name = request["name"].as<std::string>();
            if (name.length()) {
                auto iter = names_.find(name);
                if (iter != names_.end()) {
                    cell = cells_[iter->second];
                } else {
                    STENCILA_THROW(Exception, "Not found");
                }
            } else {
                STENCILA_THROW(Exception, "Either `id` or `name` parameters must be supplied");
            }
        }

        Json::Document response = Json::Object();
        response.append("id", id);
        response.append("expression", cell.expression);
        response.append("name", cell.name);
        response.append("type", cell.type);
        response.append("value", cell.value);

        return response.dump();

    } else if (verb == "PUT" and method == "evaluate") {
        auto expr = request["expr"].as<std::string>();


        auto result = evaluate(expr);

        Json::Document response = Json::Object();
        response.append("type", result[0]);
        response.append("value", result[1]);

        return response.dump();

    } else if (verb == "PUT" and method == "update") {

        if(not request.is<Json::Array>()){
            STENCILA_THROW(Exception, "Array required");
        }
        std::map<std::string,std::string> changed;
        for (unsigned int index = 0; index < request.size(); index++) {
            auto cell = request[index];
            auto id = cell["id"].as<std::string>();
            auto source = cell["source"].as<std::string>();
            changed[id] = source;
        }

        auto updates = update(changed);

        Json::Document response = Json::Array();
        for (auto update : updates) {
            Json::Document cell = Json::Object();
            cell.append("id", update.first);
            cell.append("kind", update.second[0]);
            cell.append("type", update.second[1]);
            cell.append("value", update.second[2]);
            response.append(cell);
        }
        return response.dump();

    } else if (verb == "GET" and method == "functions") {
        Json::Document response = Json::Array();
        for (auto name : functions()){
            response.append(name);
        }
        return response.dump();
    }  else if (verb == "PUT" and method == "function") {
        auto name = request["name"].as<std::string>();
        auto func = function(name);
        return func.json();
    } else {
        throw RequestInvalidException();
    }

    return "";
}


std::string Sheet::identify_row(unsigned int row) {
    return string(row+1);
}

std::string Sheet::identify_col(unsigned int col) {
    std::string id;
    while (true) {
        int mod = (col % 26) + 65;
        col /= 26;
        id = static_cast<char>(mod) + id;
        if (col > 0) col--;
        else if (col == 0) break;
    }
    return id;
}

std::string Sheet::identify(unsigned int row, unsigned int col) {
    return identify_col(col)+identify_row(row);
}

boost::regex Sheet::id_regex("^([A-Z]+)([1-9][0-9]*)$");

bool Sheet::is_id(const std::string& id){
    return boost::regex_match(id, id_regex);
}

unsigned int Sheet::index_row(const std::string& row) {
    return unstring<unsigned int>(row)-1;
}

unsigned int Sheet::index_col(const std::string& col) {
    auto index = 0u;
    auto rank = 1u;
    for(char letter : col){
        index += (col.length()-rank++)*26+(letter-65);
    }
    return index;
}

std::array<unsigned int, 2> Sheet::index(const std::string& id) {
    boost::smatch match;
    if(boost::regex_match(id, match, id_regex)){
        return {index_row(match[2]),index_col(match[1])};
    } else {
        STENCILA_THROW(Exception, "Invalid id");
    }
}

std::vector<std::string> Sheet::interpolate(
    const std::string& col1, const std::string& row1, 
    const std::string& col2, const std::string& row2
) {
    auto col1i = index_col(col1);
    auto col2i = index_col(col2);
    auto row1i = index_row(row1);
    auto row2i = index_row(row2);
    auto size = (col2i-col1i+1)*(row2i-row1i+1);
    if (size<0) STENCILA_THROW(Exception, "Invalid cell range");

    std::vector<std::string> cells(size);
    auto index = 0u;
    for (auto col = col1i; col <= col2i; col++) {
        for (auto row = row1i; row <= row2i; row++) {
            cells[index++] = identify(row,col);
        }
    }
    return cells;
}

std::array<unsigned int, 2> Sheet::extent(void) const {
    auto row = 0u;
    auto col = 0u;
    for(auto iter : cells_){
        auto indices = index(iter.first);
        row = std::max(row,indices[0]);
        col = std::max(col,indices[1]);
    }
    return {row,col};
}

Sheet& Sheet::attach(std::shared_ptr<Spread> spread) {
    spread_ = spread;
    return *this;
}

Sheet& Sheet::detach(void) {
    spread_ = nullptr;
    return *this;
}

Sheet::Cell Sheet::parse(const std::string& source) {
    Cell cell;
    if (source.length()){
        auto source_clean = source;
        boost::replace_all(source_clean, "\t", " ");

        // `\s*` at ends allows for leading and trailing spaces or newlines
        // Commented quotes below are to stop SublimeText's string formatting on subsequent line
        static const boost::regex blank_regex(R"(^\s*$)");
        
        static const std::string name = R"(^\s*([a-z]\w*)? *)";
        static const std::string expr = R"( *(.+?)\s*$)";
        static const boost::regex expression_regex(name+"\\="+expr);
        static const boost::regex mapping_regex(name+"\\~"+expr);
        static const boost::regex requirement_regex(name+"\\^"+expr);
        static const boost::regex manual_regex(name+"\\:"+expr);
        static const boost::regex test_regex(name+"\\?"+expr);

        static const boost::regex number_regex(R"(^\s*([-+]?[0-9]*\.?[0-9]+)\s*$)");
        static const boost::regex dq_string_regex(R"(^\s*("(?:[^"\\]|\\.)*")\s*$)"); // "
        static const boost::regex sq_string_regex(R"(^\s*('(?:[^"\\]|\\.)*')\s*$)"); // '

        boost::smatch match;
        if (boost::regex_match(source_clean, match, blank_regex)){
            cell.kind = Cell::blank_;
        } else if (boost::regex_match(source_clean, match, expression_regex)) {
            cell.kind = Cell::expression_;
            cell.name = match.str(1);
            cell.expression = match.str(2);
        } else if (boost::regex_match(source_clean, match, mapping_regex)) {
            cell.kind = Cell::mapping_;
            cell.name = match.str(1);
            cell.expression = match.str(2);
        } else if (boost::regex_match(source_clean, match, requirement_regex)) {
            cell.kind = Cell::requirement_;
            cell.name = match.str(1);
            cell.expression = match.str(2);
        }  else if (boost::regex_match(source_clean, match, manual_regex)) {
            cell.kind = Cell::manual_;
            cell.name = match.str(1);
            cell.expression = match.str(2);
        }  else if (boost::regex_match(source_clean, match, test_regex)) {
            cell.kind = Cell::test_;
            cell.name = match.str(1);
            cell.expression = match.str(2);
        } else if (boost::regex_match(source_clean, match, number_regex)) {
            cell.kind = Cell::number_;
            cell.expression = match.str(1);
        } else if (boost::regex_match(source_clean, match, dq_string_regex) or
                   boost::regex_match(source_clean, match, sq_string_regex)) {
            cell.kind = Cell::string_;
            cell.expression = match.str(1);
        } else {
            cell.kind = Cell::text_;
            cell.expression = "\"" + source + "\"";
        }
    }
    return cell;
}

Sheet& Sheet::source(const std::string& id, const std::string& source) {
    if (not is_id(id)) {
        STENCILA_THROW(Exception, "Not a valid cell identifier\n  id: "+id);
    }
    // Get or create the cell
    Cell& cell = cells_[id];
    // Set its attributes
    cell = parse(source);
    // Create name mapping if necessary
    if (cell.name.length()) {
        names_[cell.name] = id;
    }
    return *this;
}

std::string Sheet::source(const std::string& id) {
    const auto& iter = cells_.find(id);
    if (iter != cells_.end()) {
        const auto& cell = iter->second;
        if (cell.kind > 9) {
            return cell.expression;
        } else {
            std::string source = cell.expression;

            std::string operat;
            switch (cell.kind) {
                case Cell::expression_: operat = "="; break;
                case Cell::mapping_: operat = "~"; break;
                case Cell::requirement_: operat = "^"; break;
                case Cell::manual_: operat = ":"; break;
                case Cell::test_: operat = "?"; break;
                case Cell::visualization_: operat = "|"; break;
                default: break;
            }
            if (operat.length()) source.insert(0, operat + " ");

            if (cell.name.length()) source.insert(0, cell.name + " ");

            return source;
        }
    }
    return "";
}

std::string Sheet::translate(const std::string& expression) {
    using namespace boost::xpressive;
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");

    std::string translation;
        
    mark_tag col(1);
    mark_tag row(2);
    sregex id = (col = +range('A','Z')) >> (row = +digit);
    sregex sequence = id >> ':' >> id;
    sregex sunion = (sequence|id) >> '&' >> (sequence|id);
    sregex anything = _;
    sregex cells = sunion|sequence;
    sregex root = +(cells|anything);

    sregex_iterator iter(expression.begin(), expression.end(), root);
    sregex_iterator end;
    while (iter != end) {
        const smatch& root_expr = *iter;
        for(auto iter = root_expr.nested_results().begin(); iter != root_expr.nested_results().end(); iter++){
            const smatch& sub_expr = *iter;
            if(sub_expr.regex_id()==anything.regex_id()){
                translation += sub_expr[0];
            }
            else {
                for(auto iter = sub_expr.nested_results().begin(); iter != sub_expr.nested_results().end(); iter++){
                    const smatch& cells_expr = *iter;
                    if(cells_expr.regex_id()==sequence.regex_id()){
                        auto parts = cells_expr.nested_results().begin();
                        auto left = *(parts);
                        auto right = *(++parts);
                        auto ids = interpolate(left[col],left[row],right[col],right[row]);
                        auto combo = spread_->collect(ids);
                        translation += combo;
                    }
                    else if(cells_expr.regex_id()==sunion.regex_id()){
                        STENCILA_THROW(Exception,"Cell union operator ('&') not yet implemented");
                    }
                }
            }
        }
        ++iter;
    }

    return translation;
}

std::array<std::string, 2> Sheet::evaluate(const std::string& expression) {
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");

    // Change to the sheet's directory
    boost::filesystem::path current_path = boost::filesystem::current_path();
    boost::filesystem::path path = boost::filesystem::path(Component::path(true));
    try {
        boost::filesystem::current_path(path);
    } catch(const std::exception& exc){
        STENCILA_THROW(Exception,"Error changing to directory\n  path: "+path.string());
    }

    std::string type;
    std::string value;
    try {
        auto type_value = spread_->evaluate(translate(expression));
        auto space = type_value.find(" ");
        type = type_value.substr(0, space);
        value = type_value.substr(space+1);
    } catch (...){
        // Ensure return to current directory even if there is an exception
        boost::filesystem::current_path(current_path);
        throw;
    }
    // Return to the current directory
    boost::filesystem::current_path(current_path);

    return {type, value};
}

std::map<std::string, std::array<std::string, 3>> Sheet::update(const std::map<std::string,std::string>& changes) {
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");

    // Change to the sheet's directory
    boost::filesystem::path current_path = boost::filesystem::current_path();
    boost::filesystem::path path = boost::filesystem::path(Component::path(true));
    try {
        boost::filesystem::current_path(path);
    } catch(const std::exception& exc){
        STENCILA_THROW(Exception,"Error changing to directory\n  path: "+path.string());
    }

    // The updates resulting from the changes
    std::map<std::string, std::array<std::string, 3>> updates;
    try {
        std::vector<std::string> cells_changed;
        if (changes.size()){
            // Set the source of each cell for parsing and 
            // name mapping (required for dependency analysis below)
            for (auto iter : changes) {
                auto id = iter.first;
                auto source_ = iter.second;
                source(id,source_);
                cells_changed.push_back(id);
            }
        } else {
            // Updating source for all cells
            for (auto iter : cells_) {
                cells_changed.push_back(iter.first);
            }
        }

        // Create list of cells for which dependency needs to be updated
        // If necessary update dependency graph based on all cells
        // not just those that have been updated
        std::vector<std::string> cells_dependency;
        if (not prepared_) {
            for (const auto& iter : cells_) {
                cells_dependency.push_back(iter.first);
            }
        } else {
            cells_dependency = cells_changed;
        }

        // Function used for error checking cell access by id
        auto get_cell = [&](const std::string id) -> Cell& {
            auto iter = cells_.find(id);
            if(iter == cells_.end()) STENCILA_THROW(Exception, "Cell does not exist\n id: "+id)
            return iter->second;
        };

        // Update of dependency graph
        std::vector<std::string> cells_requirements;
        for (auto id : cells_dependency) {
            Cell& cell = get_cell(id);

            // Create vertex for the cell or clear edges for existing vertex
            Vertex vertex;
            auto iter = vertices_.find(id);
            if (iter == vertices_.end()) {
                vertex =  boost::add_vertex(id, graph_);
                vertices_[id] = vertex;
            } else {
                vertex = iter->second;
                boost::clear_in_edges(vertex, graph_);
            }
            // Requirement and manual kind cells don't need to have dependencies
            // determined
            if (cell.kind==Cell::requirement_) {
                cells_requirements.push_back(id);
            } else if (cell.kind==Cell::manual_) {

            } else {
                // Get the list of variable names this cell depends upon
                if (cell.expression.length()) {
                    auto spread_expr = translate(cell.expression);
                    // There may be a syntax error in the expression
                    // so capture those and set dependencies to none
                    std::string depends;
                    try {
                        depends = spread_->depends(spread_expr);
                    } catch(...) {
                        depends = "";
                    }
                    cell.depends.clear();
                    for (std::string depend : split(depends, ",")) {
                        if (depends.length()) {
                            // Replace cell names with cell ids
                            auto iter = names_.find(depend);
                            if (iter != names_.end()) {
                                depend = iter->second;
                            }
                            // Remove anything that is not an id (e.g. function name)
                            if (is_id(depend)) {
                                cell.depends.push_back(depend);
                            }
                        }
                    }
                } else {
                    cell.depends.clear();
                }
                // Create inward edges from cells that this one depends upon
                for (auto depend : cell.depends) {
                    Vertex vertex_from;
                    auto iter = vertices_.find(depend);
                    if (iter == vertices_.end()) {
                        vertex_from =  boost::add_vertex(depend, graph_);
                        vertices_[depend] = vertex_from;
                    } else {
                        vertex_from = iter->second;
                    }
                    boost::add_edge(vertex_from, vertex, graph_);
                }
            }
        }

        // Topological sort
        std::vector<Vertex> vertices;
        try {
            topological_sort(graph_, std::back_inserter(vertices));
        }
        catch (const std::invalid_argument& ) {
            STENCILA_THROW(Exception, "There is cyclic dependency in the sheet");
            // TODO should we create a graph visitor which shows what the cycle is?
        }
        std::vector<std::string> ids;
        for (auto vertex : vertices) {
            ids.push_back(boost::get(boost::vertex_name, graph_)[vertex]);
        }
        reverse(ids.begin(), ids.end());
        order_ = ids;

        // Next time, don't need to update all dependencies
        if (not prepared_) prepared_ = true;

        // Ensure output directory is present
        boost::filesystem::create_directories(path / "out");

        // Execute each requirement (amongst changed cells)
        for (auto id : cells_requirements) {
            Cell& cell = get_cell(id);
            spread_->execute(cell.expression);
        }

        // Iterate through order and re-execute any cell that has changed itself
        // or has predecessors that have changed 
        std::vector<std::string> cells_updated;
        for (auto id : order_) {
            // An id may exist in order_ that is not a cell (e.g. if user enters = G5 when G% is blank)
            // In that case, we don't need to do anything
            auto iter = cells_.find(id);
            if(iter == cells_.end()) continue;
            Cell& cell = iter->second;

            // Does this cell need to be executed
            // Has this cell changed?
            bool execute = std::find(cells_changed.begin(), cells_changed.end(), id) != cells_changed.end();
            if(not execute) {
                // Has any of it's immeadiate predecessors been updated?
                auto vertex = vertices_[id];
                boost::graph_traits<Graph>::in_edge_iterator edge_iter, edge_end;
                for (boost::tie(edge_iter,edge_end) = in_edges(vertex, graph_); edge_iter != edge_end; ++edge_iter) {
                    auto predecessor_vertex = boost::source(*edge_iter, graph_);
                    auto predecessor_id = boost::get(boost::vertex_name, graph_)[predecessor_vertex];
                    execute = std::find(cells_updated.begin(), cells_updated.end(), predecessor_id) != cells_updated.end();
                    if (execute) break;
                }
            }

            // If don't need to execute this cell then continue loop
            if(not execute) continue; 
            
            // Add to list of cells updated
            cells_updated.push_back(id);

            if(cell.kind == Cell::blank_){
                // If the cell source was made blank then clear it 
                // so that any dependant cells will return an error
                spread_->clear(id);
            } else if (cell.expression.length()) {
                // Store to detect any changes
                auto type = cell.type;
                auto value = cell.value;
                // Translate and execute
                auto spread_expr = translate(cell.expression);
                std::string type_value;
                try {
                    type_value = spread_->set(id, spread_expr, cell.name);
                } catch (const std::exception& exc) {
                    type_value = exc.what();
                }
                auto space = type_value.find(" ");
                cell.type = type_value.substr(0, space);
                cell.value = type_value.substr(space+1);
                // Has there been a change? Note change in kind is not detected here!
                if(cell.type != type or cell.value != value){
                    updates[id] = {
                        cell.kind_string(),
                        cell.type,
                        cell.value
                    };
                }
            }
        }
    } catch (...){
        // Ensure return to current directory even if there is an exception
        boost::filesystem::current_path(current_path);
        throw;
    }

    // Return to the current directory
    boost::filesystem::current_path(current_path);

    return updates;
}

std::map<std::string, std::array<std::string, 3>> Sheet::update(const std::string& id, const std::string& source) {
    return update({{id,source}});
}

Sheet& Sheet::update(void) {
    update({});
    return *this;
}

std::vector<std::string> Sheet::list(void) {
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");
    return split(spread_->list(), ",");
}

std::string Sheet::content(const std::string& name) {
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");
    return spread_->get(name);
}

std::vector<std::string> Sheet::depends(const std::string& id) {
    auto iter = cells_.find(id);
    if (iter == cells_.end()) {
        STENCILA_THROW(Exception, "No cell with id\n  id: "+id);
    }
    return iter->second.depends;
}

std::vector<std::string> Sheet::order(void) const {
    return order_;
}

void Sheet::graphviz(const std::string& path, bool image) const {
    std::ofstream file(path);
    boost::write_graphviz(file, graph_,
        boost::make_label_writer(boost::get(boost::vertex_name, graph_))
    );
    if(image) Helpers::execute("dot -Tpng "+path+" -o "+path+".png");
}

std::vector<std::string> Sheet::predecessors(const std::string& id) {
    auto iter = std::find(order_.begin(), order_.end(), id);
    if (iter == order_.end()) return {};
    return std::vector<std::string>(order_.begin(), iter);
}

std::vector<std::string> Sheet::successors(const std::string& id) {
    auto iter = std::find(order_.begin(), order_.end(), id);
    if (iter == order_.end()) return {};
    if (iter == order_.end()-1) return {};
    return std::vector<std::string>(iter+1, order_.end());
}

Sheet& Sheet::clear(void) {
    cells_.clear();
    names_.clear();
    graph_ = Graph();
    order_.clear();
    if (spread_) {
        spread_->clear("");
    }
    return *this;
}

std::vector<std::string> Sheet::functions(void) const {
    if (spread_) {
        return spread_->functions();
    } else {
        return {};
    }
}

Function Sheet::function(const std::string& name) const {
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");
    return spread_->function(name);
}

std::string Sheet::Cell::kind_string(void) const {
    switch (kind) {
        case Cell::blank_: return "bla";

        case Cell::expression_: return "exp";
        case Cell::mapping_: return "map";
        case Cell::requirement_: return "req";
        case Cell::manual_: return "man";
        case Cell::test_: return "tes";
        case Cell::visualization_: return "vis";

        case Cell::number_: return "num";
        case Cell::string_: return "str";
        
        case Cell::text_: return "tex";
    }
    return "";
}

}  // namespace Stencila
