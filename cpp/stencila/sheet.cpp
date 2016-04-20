#include <vector>
#include <string>
#include <algorithm>

#include <boost/algorithm/string.hpp>
#include <boost/xpressive/xpressive.hpp>
#include <boost/filesystem.hpp>
#include <boost/graph/breadth_first_search.hpp>
#include <boost/graph/graphviz.hpp>
#include <boost/graph/reverse_graph.hpp>
#include <boost/graph/topological_sort.hpp>
#include <boost/regex.hpp>

#include <stencila/debug.hpp>
#include <stencila/sheet.hpp>
#include <stencila/component-page.hpp>
#include <stencila/exception.hpp>
#include <stencila/helpers.hpp>
#include <stencila/stencil.hpp>

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
    // Look first in named cells
    auto iter = names_.find(what);
    if (iter != names_.end()){
        return cells_.at(iter->second).value;
    }
    // Otherwise in meta information
    auto meta = meta_.find(what);
    if (meta != meta_.end()){
        return meta->second;
    }
    return "";
}

std::string Sheet::environ(void) const {
    return meta("environ");
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
                    td.attr("data-display", cell.display());
                    td.text(cell.value);
                }
            }
        }
    }
    return frag;
}

Sheet& Sheet::import(const std::string& path, const std::string& at, bool execute) {
    if (not boost::filesystem::exists(path)) {
        STENCILA_THROW(Exception, "File not found\n path: "+path);
    }
    std::ifstream stream(path);
    std::string ext = boost::filesystem::extension(path);
    std::string format = ext.substr(1);
    if(format=="xlsx") {
        return load_xlsx(path, "sheet1", at, execute);
    } else {
        return load(stream, format, at);
    }
}

Sheet& Sheet::load(const std::string& string, const std::string& format, const std::string& at) {
    std::istringstream stream(string);
    return load(stream, format, at);
}

Sheet& Sheet::load(std::istream& stream, const std::string& format, const std::string& at) {
    if(format=="tsv" or format=="csv") {
        load_separated(stream, format, at);
    } else {
        STENCILA_THROW(Exception, "Format not valid for loading into a sheet\n format: "+format);
    }
    return *this;
}


Sheet& Sheet::dump_script(std::ostream& stream, const std::vector<std::string>& symbols) {
    auto assign = symbols[0];
    auto comment = symbols[1];
    auto terminate = symbols[2];

    stream << comment << " Generated by Stencila `Sheet::dump_script`\n"
           << comment << "\n"
           << comment << " Sheet address: " << address(true) << "\n"
           << comment << " Sheet cells: " << cells_.size() << "\n"
           << "\n";

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
        if (cell.kind != Cell::requirement_ and cell.expression.length()) {
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
                Cell* pointer = cell_pointer(row, col);
                if (pointer) {
                    cells[col] = pointer->source();
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
        std::string comment = "#";
        std::string termimate = "\n";
        if (format == "r") {
            assign = " <- ";
        }
        dump_script(stream, {assign, comment, termimate});
    }
    else STENCILA_THROW(Exception, "Format not valid for dumping a sheet\n format: "+format);
    return *this;
}

std::string Sheet::dump(const std::string& format) {
    std::ostringstream stream;
    dump(stream, format);
    return stream.str();
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

boost::regex read_meta("^#([\\w-]+)$");

Sheet& Sheet::read(const std::string& directory) {
    // Call base method to set component path
    Component::read(directory);
    
    // Local function for reading a TSV.
    // Mostly as per http://dataprotocols.org/linear-tsv/
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
    auto outputs = read(dir + "out/out.tsv");
    
    // Local function for getting row of outputs for a cell
    auto output = [&](const std::string& id) -> std::vector<std::string> {
        // Search for id and return column if it exists
        for(const auto& row : outputs) {
            if (row.size()) {
                if (row[0] == id) {
                    return row;
                }
            }
        }
        return {};
    };

    // Reset this sheet with new cells from source file and 
    // outputs (e.g. type, value) if they are available
    clear();
    for (const auto& row : sources) {
        auto row_size = row.size();
        if (row_size>1) {
            auto first = row[0];
            boost::smatch match;
            if (boost::regex_match(first, match, read_meta)){
                meta_[match[1]] = row[1];
            } else {
                Cell cell;
                cell.id = row[0];
                cell.source(row[1]);
                if (row_size>2) cell.display(row[2]);

                auto outs = output(cell.id);
                auto outs_size = outs.size();
                if (outs_size>1) cell.type = outs[1];
                if (outs_size>2) cell.value = outs[2];

                cells_[cell.id] = cell;
                if (cell.name.length()) names_[cell.name] = cell.id;
            }
        }
    }

    // Update dependency graph, names etc but don't execute
    update(false);

    // If a context is attached then read that too
    if(spread_) {
        spread_->read(path()+"/out/");
    }
    
    return *this;
}

Sheet& Sheet::write(const std::string& directory) {
    // Call base method to set component pth
    Component::write(directory);

    auto dir = path();
    boost::filesystem::create_directories(dir+"/out");
    std::ofstream sources(dir + "/sheet.tsv");
    std::ofstream outputs(dir + "/out/out.tsv");
    
    // Local function for escaping output.
    // Mostly as per http://dataprotocols.org/linear-tsv/
    auto escape = [](const std::string& value) {
        std::string copy = value;
        boost::replace_all(copy, "\t", "\\t");
        boost::replace_all(copy, "\n", "\\n");
        boost::replace_all(copy, "\r", "\\r");
        boost::replace_all(copy, "\\", "\\\\");
        return copy;
    };

    // Write meta
    for (auto iter : meta_) {
        sources << "#" << iter.first << "\t" << escape(iter.second) << "\n";
    }

    // Write cell sources and outputs files
    for (const auto& iter : cells_) {
        const auto& cell = iter.second;
        
        sources << cell.id << "\t" << escape(cell.source());
        auto display = cell.display_specified();
        if (display.length()) sources << "\t" << display;
        sources << "\n";

        outputs << cell.id << "\t" << escape(cell.type) << "\t" << escape(cell.value) << "\n";
    }

    // If a context is attached then write that too
    if(spread_) {
        spread_->write(dir+"/out");
    }

    return *this;
}

Sheet& Sheet::store(void) {
    write();
    Component::store();
    return *this;
}

Sheet& Sheet::restore(void) {
    Component::restore();
    read();
    return *this;
}

std::string Sheet::page(const Instance& instance) {
    return instance.as<const Sheet*>()->page();
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

std::string Sheet::request(const std::string& verb, const std::string& name, const std::string& body) {
    std::function<Json::Document(const std::string&, const Json::Document&)> callback = [&](const std::string& name, const Json::Document& args){
        return this->call(name, args);
    };
    return Component::request(verb, name, body, &callback);
}

Wamp::Message Sheet::message(const Wamp::Message& message) {
    std::function<Json::Document(const std::string&, const Json::Document&)> callback = [&](const std::string& name, const Json::Document& args){
        return this->call(name, args);
    };
    return Component::message(message, &callback);
}

Json::Document Sheet::call(const std::string& name, const Json::Document& args) {

    if(name=="write"){

        write();
        return "{}";

    } else if(name=="store"){

        store();
        return "{}";

    } else if(name=="restore"){

        restore();
        return "{}";

    } else if (name == "cell") {

        Cell cell;
        auto id = args["id"].as<std::string>();
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
            auto name = args["name"].as<std::string>();
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

        Json::Document result = Json::Object();
        result.append("id", id);
        result.append("kind", cell.kind_string());
        result.append("expression", cell.expression);
        result.append("depends", cell.depends);
        result.append("name", cell.name);
        result.append("type", cell.type);
        result.append("value", cell.value);
        result.append("display", cell.display());
        return result;

    } else if (name == "evaluate") {

        auto expr = args["expr"].as<std::string>();
        auto returned = evaluate(expr);
        Json::Document result = Json::Object();
        result.append("type", returned[0]);
        result.append("value", returned[1]);
        return result;

    } else if (name == "functions") {

        auto names = functions();
        Json::Document result = Json::Array();
        for (auto name : names){
            result.append(name);
        }
        return result;

    } else if (name == "function") {

        auto name = args[0].as<std::string>();
        auto func = function(name);
        return func.json();

    } else  if (name == "update"){

        auto arg = args[0];
        if(not arg.is<Json::Array>()){
            STENCILA_THROW(Exception, "Array required as first argument");
        }

        std::vector<Cell> changed;
        for (unsigned int index = 0; index < arg.size(); index++) {
            Cell cell;
            auto json = arg[index];
            cell.id = json["id"].as<std::string>();
            cell.source(
                json["source"].as<std::string>()
            );
            cell.display(
                json["display"].as<std::string>()
            );
            changed.push_back(cell);
        }

        std::vector<Cell> updates = update(changed);

        Json::Document result = Json::Array();
        for (auto cell : updates) {
            Json::Document json = Json::Object();
            json.append("id", cell.id);
            json.append("kind", cell.kind_string());
            json.append("type", cell.type);
            json.append("value", cell.value);
            json.append("display", cell.display());
            result.append(json);
        }
        return result;

    } else {
        return Component::call(name, args);
    }
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
boost::regex Sheet::range_regex("^([A-Z]+[1-9][0-9]*):([A-Z]+[1-9][0-9]*)$");

bool Sheet::is_id(const std::string& string){
    return boost::regex_match(string, id_regex);
}

bool Sheet::is_range(const std::string& string){
    return boost::regex_match(string, range_regex);
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
        STENCILA_THROW(Exception, "Invalid cell id\n  id: "+id);
    }
}

std::array<unsigned int, 4> Sheet::range(const std::string& range) {
    boost::smatch match;
    if(boost::regex_match(range, match, id_regex)){
        auto tl = index(range);
        return {tl[0],tl[1],tl[0],tl[1]};
    } else if (boost::regex_match(range, match, range_regex)) {
        auto tl = index(match[1]);
        auto br = index(match[2]);
        if (tl[0]>br[0] or tl[1]>br[1]) {
            STENCILA_THROW(Exception, "Invalid cell range\n  range: "+range);
        }
        return {tl[0],tl[1],br[0],br[1]};
    } else {
        STENCILA_THROW(Exception, "Invalid cell range\n  range: "+range);
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

Sheet::Cell& Sheet::cell(const std::string& id) throw(CellEmptyError) {
    auto iter = cells_.find(id);
    if (iter == cells_.end()) STENCILA_THROW(CellEmptyError, "Cell is empty\n id: "+id)
    else return iter->second;
}

Sheet::Cell& Sheet::cell(unsigned int row, unsigned int col) {
    return cell(identify(row, col));
}

Sheet::Cell* Sheet::cell_pointer(const std::string& id) {
    auto iter = cells_.find(id);
    if (iter == cells_.end()) return nullptr;
    else return &iter->second;
}

Sheet::Cell* Sheet::cell_pointer(unsigned int row, unsigned int col) {
    return cell_pointer(identify(row, col));
}


Sheet& Sheet::cells(const std::vector<std::array<std::string, 2>>& sources) {
    std::vector<Cell> cells;
    for (auto source : sources) {
        Cell cell;
        auto id = source[0];
        if (not is_id(id)) STENCILA_THROW(Exception, "Not a valid id\n id: "+id)
        cell.id = id;
        cell.source(source[1]);
        cells.push_back(cell);
    }
    clear();
    update(cells);
    return *this;
}

std::vector<Sheet::Cell> Sheet::cells(const std::string& string) {
    auto indices = range(string);
    std::vector<Cell> cells;
    for (auto row = indices[0]; row <= indices[2]; row++) {
        for (auto col = indices[1]; col <= indices[3]; col++) {
            if (auto cell = cell_pointer(identify(row,col))){
                cells.push_back(*cell);
            }
        }
    }
    return cells;
}

Sheet& Sheet::attach(std::shared_ptr<Spread> spread) {
    spread_ = spread;
    return *this;
}

Sheet& Sheet::detach(void) {
    spread_ = nullptr;
    return *this;
}

std::string Sheet::translate(const std::string& expression) {
    using namespace boost::xpressive;
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");

    std::string translation;
        
    mark_tag col(1);
    mark_tag row(2);
    sregex id = (col = +boost::xpressive::range('A','Z')) >> (row = +digit);
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

std::vector<Sheet::Cell> Sheet::update(const std::vector<Sheet::Cell>& changes, bool execute) {
    std::vector<Sheet::Cell> updates;

    // Change to the sheet's directory
    boost::filesystem::path current_path = boost::filesystem::current_path();
    boost::filesystem::path path = boost::filesystem::path(Component::path(true));
    try {
        boost::filesystem::current_path(path);
    } catch(const std::exception& exc) {
        STENCILA_THROW(Exception,"Error changing to directory\n  path: "+path.string());
    }

    try {
        std::vector<std::string> cells_changed;
        if (changes.size()) {
            // Updating only changed cells
            // Need to copy the change into the existing (or
            // newly created) cell
            for (auto cell : changes) {
                auto id = cell.id;
                Cell* pointer = cell_pointer(id);
                if (pointer) {
                    // Existing cell so copy over
                    *pointer = cell;
                    // TODO deal with any change in name by clearing the
                    // name from the context and the name mapping
                } else {
                    // New cell so insert
                    cells_.insert({id,cell});
                    // Store any name
                    if (cell.name.length()) names_[cell.name] = id;
                }
                cells_changed.push_back(id);
            }
        } else {
            // Updating all existing cells
            for (const auto& iter : cells_) {
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

        // Update of dependency graph
        std::vector<std::string> cells_requirements;
        for (auto id : cells_dependency) {
            Cell& cell = Sheet::cell(id);

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

        // If not executing (e.g. for initial read) then leave don't
        // go any further
        if (not execute) return updates;

        // Ensure output directory is present
        boost::filesystem::create_directories(path / "out");

        // Execute each requirement (amongst changed cells)
        for (auto id : cells_requirements) {
            Cell& cell = Sheet::cell(id);
            spread_->execute(cell.expression);
        }

        // Iterate through order and re-execute any cell that has changed itself
        // or has predecessors that have changed 
        std::vector<std::string> cells_updated;
        for (auto id : order_) {
            // An id may exist in order_ that is not a cell (e.g. if user enters = G5 when G5 is blank)
            // In that case, we don't need to do anything
            auto iter = cells_.find(id);
            if(iter == cells_.end()) continue;
            Cell& cell = iter->second;

            // Does this cell need to be executed...
            // ...has this cell changed?
            bool execute_cell = std::find(cells_changed.begin(), cells_changed.end(), id) != cells_changed.end();
            if(not execute_cell) {
                // ...has any of it's immeadiate predecessors been updated?
                auto vertex = vertices_[id];
                boost::graph_traits<Graph>::in_edge_iterator edge_iter, edge_end;
                for (boost::tie(edge_iter,edge_end) = in_edges(vertex, graph_); edge_iter != edge_end; ++edge_iter) {
                    auto predecessor_vertex = boost::source(*edge_iter, graph_);
                    auto predecessor_id = boost::get(boost::vertex_name, graph_)[predecessor_vertex];
                    execute_cell = std::find(cells_updated.begin(), cells_updated.end(), predecessor_id) != cells_updated.end();
                    if (execute_cell) break;
                }
            }

            // If don't need to execute this cell then continue loop
            if(not execute_cell) continue; 
            
            // Add to list of cells updated
            cells_updated.push_back(id);

            if(cell.kind == Cell::blank_) {
                // If the cell source was made blank then clear it 
                // so that any dependant cells will return an error
                spread_->clear(id);
            } else if (cell.kind == Cell::cila_) {
                // Convert source to HTML
                auto html = Stencil().cila(cell.expression).html();
                cell.value = html;
                cell.type = "html";
                updates.push_back(cell);
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
                if (cell.type != type or cell.value != value) {
                    updates.push_back(cell);
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

std::vector<Sheet::Cell> Sheet::update(const std::string& id, const std::string& source) {
    Cell cell;
    cell.id = id,
    cell.source(source);
    return update({cell});
}

Sheet& Sheet::update(const std::string& range) {
    if (range.length()==0) update();
    else update(cells(range));
    return *this;
}

Sheet& Sheet::update(bool execute) {
    update({},execute);
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


class SheetGraphvizPropertyWriter {
 public:
    SheetGraphvizPropertyWriter(const Sheet* sheet):
        sheet_(sheet) {}

    template <class VertexOrEdge>
    void operator()(std::ostream& out, const VertexOrEdge& v) const {
        auto id = boost::get(boost::vertex_name, sheet_->graph_)[v];
        const auto& cell = sheet_->cells_.at(id);

        out << "[";

        std::string label = id;
        if (cell.name.length()) label = cell.name + " (" + id + ")";
        else label = id;
        out << "label=\"" << label << "\"";

        std::string shape = "box";
        if (cell.kind >= 10) {
            shape = "circle"; 
        }
        out << ", shape=\"" << shape << "\"";

        std::string fillcolor = "whitesmoke";
        if (cell.kind == Sheet::Cell::expression_) {
            fillcolor = "aliceblue"; 
        }
        else if (cell.kind == Sheet::Cell::mapping_) {
            fillcolor = "gold"; 
        }
        else if (cell.kind == Sheet::Cell::requirement_) {
            fillcolor = "pink"; 
        }
        else if (cell.kind == Sheet::Cell::manual_) {
            fillcolor = "red"; 
        }
        else if (cell.kind == Sheet::Cell::test_) {
            fillcolor = "orange"; 
        }
        else if (cell.kind == Sheet::Cell::visualization_) {
            fillcolor = "palevioletred"; 
        }
        else if (cell.kind == Sheet::Cell::cila_) {
            fillcolor = "yellowgreen"; 
        }
        else if (cell.kind >= 10) {
            fillcolor = "gray"; 
        }
        out << ", style=\"filled\", fillcolor=\"" << fillcolor << "\"";

        out << "]";
    }
 private:
    const Sheet* sheet_;
};

void Sheet::graphviz(const std::string& filepath, bool image) const {
    std::string dot_filepath = filepath;
    if (not dot_filepath.length()) {
        auto p = path(true);
        boost::filesystem::create_directories(p + "/out");
        dot_filepath = p + "/out/graph.dot";
    }
    std::ofstream dot_file(dot_filepath);
    boost::write_graphviz(dot_file, graph_, SheetGraphvizPropertyWriter(this));
    if(image) {
        boost::filesystem::path dot_filepath_p(dot_filepath);
        std::string png_filepath = (
            dot_filepath_p.parent_path() / 
            dot_filepath_p.stem()
        ).string() + ".png";
        Helpers::execute("dot -Tpng "+dot_filepath+" -o "+png_filepath);
    }
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

template<class Predecessors>
class test_bfs_visitor : public boost::default_bfs_visitor {
  public:
    Predecessors& predecessors;

    test_bfs_visitor(Predecessors& predecessors_):
        predecessors(predecessors_){}

    template <typename Vertex, typename Graph>
    void discover_vertex(const Vertex& v, Graph& g) {
      predecessors.push_back(v);
    }
};

Json::Document Sheet::test(void) const {
    // Reverse graph for predecessor analysis
    auto reversed_graph = boost::make_reverse_graph(graph_);

    Json::Document results;
    int cells = 0;
    int expressions = 0;
    int tests = 0;
    int passes = 0;
    int fails = 0;
    int errors = 0;
    int covered = 0;
    for(const auto& iter : cells_) {
        const auto& id = iter.first;
        const auto& cell = iter.second;
        cells++;
        if (cell.kind == Cell::test_) {
            tests++;
            if (cell.value == "true") {
                passes++;
            }
            else if (cell.value == "false") {
                fails++;
            }
            else if (cell.type == "error") {
                errors++;
            }

            // Count test predecessors that are expressions
            std::vector<Vertex> predecessors;
            test_bfs_visitor<decltype(predecessors)> visitor(predecessors);
            boost::breadth_first_search(
                reversed_graph,
                vertices_.at(id),
                boost::visitor(visitor)
            );
            for(const auto& predecessor_vertex : predecessors) {
                auto predecessor_id = boost::get(boost::vertex_name, reversed_graph)[predecessor_vertex];
                if (predecessor_id == id) continue;
                const auto& predecessor = cells_.at(predecessor_id);
                if (predecessor.kind == Cell::expression_) {
                    covered++;
                }
            }
        } else if (cell.kind == Cell::expression_) {
            expressions++;
        }
    }
    results.append("cells", cells);
    results.append("expressions", expressions);
    results.append("tests", tests);
    results.append("passes", passes);
    results.append("fails", fails);
    results.append("errors", errors);
    results.append("covered", covered);
    results.append("coverage", expressions>0?covered/double(expressions):0);
    return results;
}

Sheet& Sheet::clear(void) {
    cells_.clear();
    names_.clear();
    vertices_.clear(); // Container pointers to graph_ vertices, so clear first!
    graph_.clear();
    order_.clear();
    prepared_ = false;
    if (spread_) {
        spread_->clear();
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
        case Cell::cila_: return "cil";

        case Cell::number_: return "num";
        case Cell::string_: return "str";
        
        case Cell::text_: return "tex";
    }
    return "";
}

std::string Sheet::Cell::source(void) const {
    if (kind > 9) {
        return expression;
    } else {
        std::string source = expression;

        std::string operat;
        switch (kind) {
            case expression_: operat = "="; break;
            case mapping_: operat = "~"; break;
            case requirement_: operat = "^"; break;
            case manual_: operat = ":"; break;
            case test_: operat = "?"; break;
            case visualization_: operat = "|"; break;
            case cila_: operat = "_"; break;
            default: break;
        }
        if (operat.length()) source.insert(0, operat + " ");

        if (name.length()) source.insert(0, name + " ");

        return source;
    }
}

Sheet::Cell& Sheet::Cell::source(const std::string& source) {
    auto source_clean = source;
    boost::replace_all(source_clean, "\t", " ");

    // `\s*` at ends allows for leading and trailing spaces or newlines
    // Commented quotes below are to stop SublimeText's string formatting on subsequent line
    static const boost::regex blank_regex(R"(^\s*$)");
    
    static const std::string name_re = R"(^\s*([a-z]\w*)? *)";
    static const std::string expr_re = R"( *(.+?)\s*$)";
    static const boost::regex expression_regex(name_re+"\\="+expr_re);
    static const boost::regex mapping_regex(name_re+"\\:"+expr_re);
    static const boost::regex requirement_regex(name_re+"\\^"+expr_re);
    static const boost::regex manual_regex(name_re+"\\|"+expr_re);
    static const boost::regex test_regex(name_re+"\\?"+expr_re);
    static const boost::regex visualization_regex(name_re+"\\~"+expr_re);
    static const boost::regex cila_regex(name_re+"\\_"+expr_re);

    static const boost::regex number_regex(R"(^\s*([-+]?[0-9]*\.?[0-9]+)\s*$)");
    static const boost::regex dq_string_regex(R"(^\s*("(?:[^"\\]|\\.)*")\s*$)"); // "
    static const boost::regex sq_string_regex(R"(^\s*('(?:[^"\\]|\\.)*')\s*$)"); // '

    boost::smatch match;
    if (boost::regex_match(source_clean, match, blank_regex)){
        kind = Cell::blank_;
    } else if (boost::regex_match(source_clean, match, expression_regex)) {
        kind = Cell::expression_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, mapping_regex)) {
        kind = Cell::mapping_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, requirement_regex)) {
        kind = Cell::requirement_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, manual_regex)) {
        kind = Cell::manual_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, test_regex)) {
        kind = Cell::test_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, visualization_regex)) {
        kind = Cell::visualization_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, cila_regex)) {
        kind = Cell::cila_;
        name = match.str(1);
        expression = match.str(2);
    } else if (boost::regex_match(source_clean, match, number_regex)) {
        kind = Cell::number_;
        expression = match.str(1);
    } else if (boost::regex_match(source_clean, match, dq_string_regex) or
               boost::regex_match(source_clean, match, sq_string_regex)) {
        kind = Cell::string_;
        expression = match.str(1);
    } else {
        kind = Cell::text_;
        expression = "\"" + source + "\"";
    }

    return *this;
}

std::string Sheet::Cell::display(void) const {
    if (display_.length()){
        return display_;
    } else {
        if (type=="image_file" or type=="html") {
            return "ove";
        } else if(type=="error") {
            return "exp";
        } else {
            return "cli";
        }
    }
}

std::string Sheet::Cell::display_specified(void) const {
    return display_;
}

Sheet::Cell& Sheet::Cell::display(const std::string& display) {
    display_ = display;
    return *this;
}

}  // namespace Stencila
