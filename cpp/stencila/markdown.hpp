#pragma once

#include <string>

#include <stencila/xml.hpp>

struct cmark_node;

namespace Stencila {
namespace Markdown {

/**
 * A Markdown document
 *
 * Currently implemented via `cmark` (https://github.com/jgm/cmark). As such, this class's method 
 * interface is largely determined by `cmark`s current functionality.
 */
class Document {
 public:
  /**
   * Construct a Markdown document
   */
  explicit Document(const std::string& md = "");

  /**
   * Destroy a Markdown document
   */
  ~Document(void);

  /**
   * Set content from a Markdown string
   * 
   * @param  md Markdown string 
   */
  Document& md(const std::string& md);

  /**
   * Get content as a Markdown string 
   */
  std::string md(int width = 0) const;

  /**
   * Get content as a HTML string
   */
  std::string html(void) const;

  /**
   * Get content as a HTML document
   */
  Xml::Document html_doc(void) const;

  /**
   * Set content from a HTML document
   * 
   * @param  doc Document (actually an `Xml::Document` at present)
   */
  Document& html_doc(const Xml::Document& doc);

  /**
   * Get content as a Latex string
   */
  std::string latex(int width = 100) const;

  /**
   * Get content as a groff man string
   */
  std::string man(int width = 100) const;

  /**
   * Read the document from a file
   * 
   * @param  path File system path
   */
  Document& read(const std::string& path);

  /**
   * Write the document to a file
   * 
   * @param  path File system path
   */
  Document& write(const std::string& path, const std::string& format = "");

 protected:
    cmark_node* root_ = nullptr;
};

}
}
