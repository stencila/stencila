#include <stencila/debug.hpp>
#include <stencila/json.hpp>
#include <stencila/markdown.hpp>
#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::jnb(void) const {
  return "";
}

Stencil& Stencil::jnb(const std::string& jnb) {
  Json::Document json(jnb);
  // TODO : get ["metadata"]["language_info"]
  if (not json.has("cells")) STENCILA_THROW(Exception, "Missing 'cells'");
  auto cells = json["cells"];
  for (int index=0; index<cells.size(); index++) {
    auto cell = cells[index];
    auto cell_type = cell.get<std::string>("cell_type");
    auto source = cell.get<std::string>("source");
    if (cell_type == "markdown") {
      append(Markdown::Document(source).html_doc());
    }
    else if (cell_type == "code") {
      append("pre", {{"data-exec", "r"}}, source);
    }
  }
  return *this;
}

} //namespace Stencila
