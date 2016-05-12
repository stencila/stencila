#include <fstream>
#include <string>

#include <boost/filesystem.hpp>

#include <cmark.h>

#include <stencila/exception.hpp>
#include <stencila/markdown.hpp>

namespace Stencila {
namespace Markdown {

Document::Document(const std::string& content) {
  md(content);
}

Document::~Document(void) {
  if (root_) cmark_node_free(root_);
}

Document& Document::md(const std::string& md) {
  if (root_) cmark_node_free(root_);
  root_ = cmark_parse_document(md.c_str(), md.length(), CMARK_OPT_DEFAULT);
  return *this;
}

namespace {
  // Garbage collection used in following methods
  std::string wrap(char* chars) {
    std::string string = chars;
    free(chars);
    return string;
  }
}

std::string Document::md(int width) const {
  return root_?wrap(cmark_render_commonmark(root_, CMARK_OPT_DEFAULT, width)):"";
}

std::string Document::html(void) const {
  return root_?wrap(cmark_render_html(root_, CMARK_OPT_DEFAULT)):"";
}

std::string Document::latex(int width) const {
  return root_?wrap(cmark_render_latex(root_, CMARK_OPT_DEFAULT, width)):"";
}

std::string Document::man(int width) const {
  return root_?wrap(cmark_render_man(root_, CMARK_OPT_DEFAULT, width)):"";
}

Document& Document::read(const std::string& path) {
  if (not boost::filesystem::exists(path)) {
    STENCILA_THROW(Exception, "File not found at path\n  path: " + path);
  }

  if (root_) cmark_node_free(root_);
  FILE* file = fopen(path.c_str(), "rb");
  root_ = cmark_parse_file(file, CMARK_OPT_DEFAULT);
  return *this;
}

Document& Document::write(const std::string& path, const std::string& format) {
  std::string format_ = format;
  if (format_ == "") {
    format_ = boost::filesystem::extension(path).substr(1);
  }
  std::ofstream file(path);
  if (format_ == "html") {
    file << html();
  } else if (format_ == "latex") {
    file << html();
  } else if (format_ == "man" or format_ == "groff" or format_ == "roff") {
    file << man();
  } else {
    file << md();
  }
  return *this;
}

}
}
