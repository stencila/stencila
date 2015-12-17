#include <vector>
#include <string>

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
    // TODO
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
            auto iter = cells_.find(id);
            if (iter != cells_.end()) {
                auto& cell = iter->second;
                if (cell.expression.length()) td.attr("data-expr", cell.expression);
                if (cell.name.length()) td.attr("data-name", cell.name);
                if (cell.type.length()) td.attr("data-type", cell.type);
                td.text(cell.value);
            }
        }
    }
    return frag;
}

Sheet& Sheet::load(std::istream& stream, const std::string& format) {
    clear();
    unsigned int row = 0;
    std::string line;
    while (std::getline(stream, line)) {
        std::vector<std::string> cells;
        boost::split(cells, line, boost::is_any_of("\t"));
        unsigned int col = 0;
        for (auto cell : cells) {
            if (cell.length()) {
                content(identify(row, col), cell);
            }
            col++;
        }
        row++;
    }
    return *this;
}

Sheet& Sheet::load(const std::string& string, const std::string& format) {
    std::istringstream stream(string);
    return load(stream, format);
}

Sheet& Sheet::dump(std::ostream& stream, const std::string& format) {
    if (format == "tsv") {
        auto extents = extent();
        auto rows = extents[0];
        auto cols = extents[1];
        for (unsigned int row = 0; row < rows; row++) {
            for (unsigned int col = 0; col < cols; col++) {
                auto id = identify(row, col);
                auto iter = cells_.find(id);
                if (iter != cells_.end()) {
                    auto& cell = iter->second;
                    stream << cell.value;
                }
                stream << "\t";
            }
            stream << "\n";
        }
    }
    else STENCILA_THROW(Exception, "File extension not valid for a sheet\n extension: "+format);
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
    if (ext == ".tsv") {
        std::ofstream file(path);
        dump(file, "tsv");
    }
    else STENCILA_THROW(Exception, "File extension not valid for a sheet\n extension: "+ext);
    return *this;
}

Sheet& Sheet::read(const std::string& directory) {
    Component::read(directory);
    import(path()+"/sheet.tsv");
    return *this;
}

Sheet& Sheet::write(const std::string& directory) {
    // TODO
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
    // Number of rows and columns should be neither too small not tool large
    auto extents = extent();
    auto rows = extents[0];
    rows = std::min(std::max(rows+1, 50u), 200u);
    auto cols = extents[1];
    cols = std::min(std::max(cols+1, 20u), 100u);
    main.append(html_table(rows, cols));

    return doc.dump(false);
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

    } else if (verb == "PUT" and method == "update") {

        if(not request.is<Json::Array>()){
            STENCILA_THROW(Exception, "Array required");
        }
        std::map<std::string,std::string> changed;
        for (unsigned int index = 0; index < request.size(); index++) {
            auto cell = request[index];
            auto id = cell["id"].as<std::string>();
            auto content = cell["content"].as<std::string>();
            changed[id] = content;
        }

        auto updates = update(changed);

        Json::Document response = Json::Array();
        for (auto update : updates) {
            Json::Document cell = Json::Object();
            cell.append("id", update.first);
            cell.append("type", update.second[0]);
            cell.append("value", update.second[1]);
            response.append(cell);
        }
        return response.dump();

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
    // TODO
    return {10,10};
}

Sheet& Sheet::attach(std::shared_ptr<Spread> spread) {
    spread_ = spread;
    return *this;
}

Sheet& Sheet::detach(void) {
    spread_ = nullptr;
    return *this;
}

std::array<std::string, 2> Sheet::parse(const std::string& content) {
    auto content_clean = content;
    boost::replace_all(content_clean, "\t", " ");

    static const boost::regex regex("^ *(([a-z]\\w*) *= *)?(.+?) *$");
    boost::smatch match;
    if (boost::regex_match(content_clean, match, regex)) {
        return {match.str(3), match.str(2)};
    } else {
        return {"", ""};
    }
}

Sheet& Sheet::content(const std::string& id, const std::string& content) {
    if (not is_id(id)) {
        STENCILA_THROW(Exception, "Not a valid cell identifier\n  id: "+id);
    }
    if (content.length()) {
        // Get or create the cell
        Cell& cell = cells_[id];
        // Set its attributes
        if (content.length()) {
            auto parts = parse(content);
            cell.expression = parts[0];
            cell.name = parts[1];
        }
        // Create name mapping if necessary
        if (cell.name.length()) {
            names_[cell.name] = id;
        }
    } else {
        // Clear the cell
        clear(id);
    }
    return *this;
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

std::map<std::string, std::array<std::string, 2>> Sheet::update(const std::map<std::string,std::string>& cells) {
    // Change to the sheet's directory
    boost::filesystem::path current_path = boost::filesystem::current_path();
    boost::filesystem::path path = boost::filesystem::path(Component::path(true));
    try {
        boost::filesystem::current_path(path);
    } catch(const std::exception& exc){
        STENCILA_THROW(Exception,"Error changing to directory\n  path: "+path.string());
    }

    std::vector<std::string> changed;
    std::map<std::string, std::array<std::string, 2>> updates;
    try {

        if (cells.size()){
            // First pass: set the content of each cell for name mapping (required for dependency analysis)
            for (auto iter : cells) {
                auto id = iter.first;
                auto contnt = iter.second;
                content(id,contnt);
                // Keep track of those with content (as opposed to those wich were cleared because content=="")
                if (contnt.length()) changed.push_back(id);
            }
        } else {
            // Updating all cells
            for (auto iter : cells_) changed.push_back(iter.first);
        }

        // Second pass: updating of dependency graph
        for (auto id : changed) {
            // Get the cell
            Cell& cell = cells_.at(id);
            // Get the list of variable names this cell depends upon
            if (cell.expression.length() and spread_) {
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
            // Create vertex, or clear edges for existing vertex, in the dependency graph
            Vertex vertex;
            auto iter = vertices_.find(id);
            if (iter == vertices_.end()) {
                vertex =  boost::add_vertex(id, graph_);
                vertices_[id] = vertex;
            } else {
                vertex = iter->second;
                boost::clear_in_edges(vertex, graph_);
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

        // Topological sort to determine recalculation order
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

        // Iterate through topological sort order and once the first changed
        // cell is hit, start recalculating subsequent cells
        bool calculate = false;
        for (auto id : order_) {
            if (not calculate) {
                if (std::find(changed.begin(), changed.end(), id) != changed.end()) {
                    calculate = true;
                }
            }
            if (calculate) {
                auto iter = cells_.find(id);
                if(iter == cells_.end()) STENCILA_THROW(Exception, "Cell does not exist\n id: "+id)
                Cell& cell = iter->second;
                if (cell.expression.length() and spread_) {
                    auto spread_expr = translate(cell.expression);
                    std::string type_value;
                    try {
                        type_value = spread_->set(id, spread_expr, cell.name);
                    } catch (const std::exception& exc) {
                        type_value = exc.what();
                    }
                    auto space = type_value.find(" ");
                    cell.type = type_value.substr(0,space);
                    cell.value = type_value.substr(space+1);
                    updates[id] = {
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

std::map<std::string, std::array<std::string, 2>> Sheet::update(const std::string& id, const std::string& content) {
    return update({{id,content}});
}

Sheet& Sheet::update(void) {
    update({});
    return *this;
}

std::vector<std::string> Sheet::list(void) {
    if (not spread_) STENCILA_THROW(Exception, "No spread attached to this sheet");
    return split(spread_->list(), ",");
}

std::string Sheet::value(const std::string& name) {
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

std::vector<std::string> Sheet::order(void) {
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

Sheet& Sheet::clear(const std::string& id) {
    cells_.erase(id);
    auto name = names_.find(id);
    if(name != names_.end()){
        names_.erase(name->second);
    }
    auto vertex = vertices_.find(id);
    if(vertex != vertices_.end()){
        boost::remove_vertex(vertex->second,graph_);
    }
    if (spread_) {
        spread_->clear(id);
    }
    return *this;
}

Sheet& Sheet::clear(void) {
    cells_.clear();
    names_.clear();
    graph_ = Graph();
    if (spread_) {
        spread_->clear("");
    }
    return *this;
}

}  // namespace Stencila
