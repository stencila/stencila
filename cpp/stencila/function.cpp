#include <vector>
#include <string>
#include <algorithm>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/regex.hpp>

#include <stencila/function.hpp>
#include <stencila/component-page.hpp>
#include <stencila/exception.hpp>
#include <stencila/helpers.hpp>

namespace Stencila {

Function::Function(void) {
}

Function::Function(const std::string& from) {
    initialise(from);
}

Function::~Function(void) {
}

Component::Type Function::type(void) {
    return FunctionType;
}

std::string Function::meta(const std::string& what) const {
    return "";
}

std::string Function::title(void) const {
    return meta("title");
}

std::string Function::description(void) const {
    return meta("description");
}

std::vector<std::string> Function::keywords(void) const {
    auto content = meta("keywords");
    if (content.length()) {
        auto values = split(content, ",");
        for (auto& value : values) trim(value);
        return values;
    } else {
        return {};
    }
}

std::vector<std::string> Function::authors(void) const {
    auto content = meta("authors");
    if (content.length()) {
        auto values = split(content, ",");
        for (auto& value : values) trim(value);
        return values;
    } else {
        return {};
    }
}

std::string Function::theme(void) const {
    return "";
}


Function& Function::initialise(const std::string& from) {
    if (boost::filesystem::exists(from)) {
        read(from);
    } else {
        std::string path = Component::locate(from);
        if (path.length()) read(path);
        else STENCILA_THROW(Exception, "No function found with path or address:\n path: "+from);
    }
    return *this;
}

Function& Function::load(std::istream& stream, const std::string& format) {
    if (format == "yaml") {
    }
    else STENCILA_THROW(Exception, "File extension not valid for loading a sheet\n extension: "+format);
    return *this;
}

Function& Function::load(const std::string& string, const std::string& format) {
    std::istringstream stream(string);
    return load(stream, format);
}

Function& Function::dump(std::ostream& stream, const std::string& format) {
    if (format == "yaml") {
    } else if (format == "r" or format=="py") {
    }
    else STENCILA_THROW(Exception, "Format not valid for dumping a function\n format: "+format);
    return *this;
}

std::string Function::dump(const std::string& format) {
    std::ostringstream stream;
    dump(stream, format);
    return stream.str();
}

Function& Function::import(const std::string& path) {
    if (not boost::filesystem::exists(path)) {
        STENCILA_THROW(Exception, "File not found\n path: "+path);
    }
    std::string ext = boost::filesystem::extension(path);
    if (ext == ".tsv") {
        std::ifstream file(path);
        load(file, "tsv");
    }
    else STENCILA_THROW(Exception, "File extension not valid for a function\n extension: "+ext);
    return *this;
}

Function& Function::export_(const std::string& path) {
    std::string ext = boost::filesystem::extension(path);
    if (ext == ".tsv" or ext == ".r" or ext == ".py") {
        std::ofstream file(path);
        auto format = ext.substr(1);
        dump(file, format);
    }
    else STENCILA_THROW(Exception, "File extension not valid for a function\n extension: "+ext);
    return *this;
}

Function& Function::read(const std::string& directory) {
    // Call base method to set component path
    Component::read(directory);
 
    return *this;
}

Function& Function::write(const std::string& directory) {
    // Call base method to set component pth
    Component::write(directory);


    return *this;
}

std::string Function::page(const Component* component) {
    return static_cast<const Function&>(*component).page();
}

std::string Function::page(void) const {
    // Get base document
    Html::Document doc = Component_page_doc<Function>(*this);
    Html::Node head = doc.find("head");
    Html::Node body = doc.find("body");

    // Add to main#content
    auto main = body.select("main");
    main.attr("id", "content");
    // TODO

    return doc.dump(false);
}

Function& Function::compile(void) {
    auto home = boost::filesystem::path(path(true));
    auto filepath = (home/"index.html").string();
    std::ofstream file(filepath);
    file << page();
    return *this;
}

std::string Function::serve(void) {
    return Component::serve(FunctionType);
}

Function& Function::view(void) {
    Component::view(FunctionType);
    return *this;
}

std::string Function::request(Component* component, const std::string& verb, const std::string& method, const std::string& body) {
    return static_cast<Function&>(*component).request(verb, method, body);
}

std::string Function::request(const std::string& verb, const std::string& method, const std::string& body) {
    Json::Document request;
    if (body.length()) {
        request.load(body);
    }

    return "";
}

Function& Function::attach(std::shared_ptr<Context> context) {
    context_ = context;
    return *this;
}

Function& Function::detach(void) {
    context_ = nullptr;
    return *this;
}

}  // namespace Stencila
