#include <vector>
#include <string>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/graph/topological_sort.hpp>
#include <boost/regex.hpp>

#include <stencila/sheet.hpp>
#include <stencila/component-page.hpp>
#include <stencila/exception.hpp>

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
        if (cell.alias == what) {
            return cell.expression;
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
                td.text(cell.value);
                if (cell.expression.length()) td.attr("data-expr", cell.expression);
                if (cell.alias.length()) td.attr("data-alias", cell.alias);
            }
        }
    }
    return frag;
}

Sheet& Sheet::load(std::istream& stream, const std::string& format) {
    unsigned int row = 0;
    std::string line;
    while (std::getline(stream, line)) {
        std::vector<std::string> cells;
        boost::split(cells, line, boost::is_any_of("\t"));
        unsigned int col = 0;
        for (auto content : cells) {
            // TODO, the initial execution of the cell's
            // should not be done until order_ is determined
            update(identify(row, col), content);
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
        // TODO rows, cols
        uint rows = 10;
        uint cols = 10;
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
    main.append(html_table());

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
    Json::Document response = Json::Object();

    if (verb == "PUT" and method == "update") {
        for (auto iter = request.begin(); iter != request.end();) {
            auto id = iter.key().as<std::string>();
            auto source = (*iter).as<std::string>();
            auto new_value = update(id, source);
            response.append(id, new_value);
            ++iter;
        }
    }
    else {
        throw RequestInvalidException();
    }

    return response.dump();
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

Sheet& Sheet::attach(std::shared_ptr<Spread> spread) {
    spread_ = spread;
    return *this;
}

Sheet& Sheet::detach(void) {
    spread_ = nullptr;
    return *this;
}

std::array<std::string, 3> Sheet::parse(const std::string& content) {
    auto content_clean = content;
    boost::replace_all(content_clean, "\t", " ");

    static const boost::regex regex("^ *([a-z]\\w*)? *= *(.+?) *$");
    boost::smatch match;
    if (boost::regex_match(content_clean, match, regex)) {
        return {"", match.str(2), match.str(1)};
    } else {
        return {content_clean, "", ""};
    }
}

std::string Sheet::update(const std::string& id, const std::string& content) {
    // Get or create the cell
    Cell& cell = cells_[id];
    // Set its attributes
    if (content.length()) {
        auto parts = parse(content);
        cell.value = parts[0];
        cell.expression = parts[1];
        cell.alias = parts[2];
    }
    // Create alias mapping if necessary
    if (cell.alias.length()) {
        aliases_[cell.alias] = id;
    }
    if (spread_) {
        // Set cell value
        auto expr = cell.expression.length()?cell.expression:cell.value;
        if (expr.length()) {
            cell.value = spread_->set(id, expr, cell.alias);
        }
        if (cell.expression.length()) {
            // Get the list of variable names this cell depends upon
            cell.depends = split(spread_->depends(cell.expression), ",");
            // Replace cell aliases with cell ids
            for (std::string& depend : cell.depends) {
                auto iter = aliases_.find(depend);
                if (iter != aliases_.end()) {
                    depend = iter->second;
                }
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
        boost::clear_vertex(vertex, graph_);
    }
    // Create inward edges from cells that this one dependes upon
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
    // Do topological sort to determine update order
    std::vector<Vertex> vertices;
    try {
        topological_sort(graph_, std::back_inserter(vertices));
    }
    catch (const std::invalid_argument& ){
        STENCILA_THROW(Exception, "There is cyclic dependency in cell\n id: "+id);
    }
    std::vector<std::string> ids;
    for (auto vertex : vertices) {
        ids.push_back(boost::get(boost::vertex_name, graph_)[vertex]);
    }
    reverse(ids.begin(), ids.end());
    order_ = ids;

    return cell.value;
}

Sheet& Sheet::update(void) {
    for (std::map<std::string,Cell>::iterator iter = cells_.begin(); iter != cells_.end(); iter++) {
        update(iter->first);
    }
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
    auto& cell = cells_[id];
    cells_.erase(id);
    aliases_.erase(cell.alias);
    // TODO remove vertex from graph: 
    if (spread_) {
        spread_->clear(id);
    }
    return *this;
}

Sheet& Sheet::clear(void) {
    cells_.clear();
    aliases_.clear();
    // TODO clear graph: graph_ = Graph();
    if (spread_) {
        spread_->clear("");
    }
    return *this;
}

}  // namespace Stencila
