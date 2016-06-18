#include <string>

#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::schema(void) const {
  return schema_;
}


Stencil& Stencil::schema(const std::string& schema) {
  if (schema == "" or schema == "rmd") {
    schema_ = schema;
  } else {
    STENCILA_THROW(Exception, "Invalid schema\n  schema: " + schema);
  }
  return *this;
}


Stencil& Stencil::conform(const std::string& _schema) {
  schema(_schema);
  // Currently just conforming to default schema...
  // others still TODO
  if (schema_ == "") {
    // Ensure no orphan text nodes at the top level
    for (auto child : children()) {
      if (child.is_text()) {
        child.replace("p", {}, child.text());
      }
    }
  }
  return *this;
}

}  // namespace Stencila
