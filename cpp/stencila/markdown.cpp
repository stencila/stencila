#include <fstream>
#include <string>

#include <boost/filesystem.hpp>

#include <stencila/exception.hpp>
#include <stencila/debug.hpp>
#include <stencila/html.hpp>
#include <stencila/markdown.hpp>
#include <stencila/string.hpp>
#include <stencila/xml.hpp>

#if defined(_WIN32)
	// Tell cmark we will link to it statically
	#define CMARK_STATIC_DEFINE
#endif
#include <cmark.h>

namespace Stencila {
namespace Markdown {

Document::Document(const std::string& content) {
  md(content);
}

Document::~Document(void) {
  if (root_) cmark_node_free(root_);
}

// Local helper functions
namespace {

  // Garbage collection used to wrap string generating
  // rendering functions below
  std::string wrap_render(char* chars) {
    std::string string = chars;
    free(chars);
    return string;
  }

  // Building a XML document from cmark nodes
  void build_html_tree(Xml::Node parent, cmark_node* cnode) {
    Xml::Node node;

    cmark_node_type cnode_type = cmark_node_get_type(cnode);
    switch (cnode_type) {
      case CMARK_NODE_DOCUMENT:
        node = parent;
      break;

      case CMARK_NODE_BLOCK_QUOTE:
        node = parent.append("blockquote");
      break;

      case CMARK_NODE_LIST:
      {
        auto list_type = cmark_node_get_list_type(cnode);
        if (list_type == CMARK_ORDERED_LIST) {
          node = parent.append("ol");
        } else {
          node = parent.append("ul");
        }
      }
      break;

      case CMARK_NODE_ITEM:
        node = parent.append("li");
      break;

      case CMARK_NODE_CODE_BLOCK:
      {
        auto pre = parent.append("pre");
        node = pre.append("code");
        std::string info = cmark_node_get_fence_info(cnode);
        if (info.length()) node.attr("class", info);
        node.text(cmark_node_get_literal(cnode));
      }
      break;

      case CMARK_NODE_PARAGRAPH:
      {
        // Unwrap paragraph nodes that are below `blockquote` and `li`
        if (parent.name()=="blockquote" or parent.name()=="li") {
          node = parent;
        } else {
          node = parent.append("p");
        }
      }
      break;

      case CMARK_NODE_HEADING:
      {
        auto level = cmark_node_get_heading_level(cnode);
        node = parent.append("h"+string(level));
      }
      break;
      
      case CMARK_NODE_TEXT:
        parent.append_text(cmark_node_get_literal(cnode));
      break;

      case CMARK_NODE_CODE:
        node = parent.append("code");
        node.text(cmark_node_get_literal(cnode));
      break;

      case CMARK_NODE_EMPH:
        node = parent.append("em");
      break;

      case CMARK_NODE_STRONG:
        node = parent.append("strong");
      break;

      case CMARK_NODE_LINK:
      case CMARK_NODE_IMAGE:
      {
        std::string url_attr;
        if (cnode_type == CMARK_NODE_LINK) {
          node = parent.append("a");
          url_attr = "href";
        } else {
          node = parent.append("img");
          url_attr = "src";
        }
        std::string url = cmark_node_get_url(cnode);
        if (url.length()) {
          node.attr(url_attr, url);
        }
        std::string title = cmark_node_get_title(cnode);
        if (title.length()) {
          node.attr("title", title);
        }
      }
      break;

      case CMARK_NODE_HTML_BLOCK:
      case CMARK_NODE_HTML_INLINE:
        parent.append_xml(cmark_node_get_literal(cnode));
      break;

      default:
        // Fallback for node types not yet handled above so that 
        // content is not lost
        node = parent.append("div");
      break;
    }

    if (node) {
      cmark_node* child = cmark_node_first_child(cnode);
      cmark_node* last = cmark_node_last_child(cnode);
      while (child) {
        build_html_tree(node, child);
        if (child == last) break;
        child = cmark_node_next(child);
      }
    }
  }

  // Building a cmark document from XML nodes 
  void build_cmark_tree(cmark_node* parent, Xml::Node xnode) {
    cmark_node* node;
    if (xnode.is_document()) {
      for (auto child : xnode.children()) build_cmark_tree(parent, child);
      return;
    }
    else if (xnode.is_text()) {
      node = cmark_node_new(CMARK_NODE_TEXT);
      cmark_node_set_literal(node, xnode.text().c_str());
    } 
    else {
      bool build_children = true;
      auto tag = xnode.name();
      if (tag == "blockquote") {
        node = cmark_node_new(CMARK_NODE_BLOCK_QUOTE);
      }
      else if (tag == "ul" or tag == "ol") {
        node = cmark_node_new(CMARK_NODE_LIST);
        cmark_node_set_list_type(node, (tag == "ol")?CMARK_ORDERED_LIST:CMARK_BULLET_LIST);
        cmark_node_set_list_tight(node, 1);
      }
      else if (tag == "li") {
        node = cmark_node_new(CMARK_NODE_ITEM);
      }
      else if (tag == "pre") {
        auto code = xnode.find("code");
        if (code) {
          node = cmark_node_new(CMARK_NODE_CODE_BLOCK);
          cmark_node_set_literal(node, code.text().c_str());
          // The Commonmark spec 0.25 says:
          //     The first word of the info string is typically used to specify the 
          //     language of the code sample, and rendered in the class attribute of 
          //     the code tag. However, this spec does not mandate any particular 
          //     treatment of the info string.
          if (code.has("class")) {
            cmark_node_set_fence_info(node, code.attr("class").c_str());
          }
        } else {
          node = cmark_node_new(CMARK_NODE_HTML_INLINE);
          cmark_node_set_literal(node, xnode.dump().c_str());
        }
        build_children = false;
      }
      else if (tag == "h1" or tag == "h2" or tag == "h3" or tag == "h4" or tag == "h5" or tag == "h6") {
        node = cmark_node_new(CMARK_NODE_HEADING);
        auto level = unstring<int>(tag.substr(1));
        cmark_node_set_heading_level(node, level);
      }
      else if (tag == "p") {
        node = cmark_node_new(CMARK_NODE_PARAGRAPH);
      }
      else if (tag == "code") {
        node = cmark_node_new(CMARK_NODE_CODE);
        cmark_node_set_literal(node, xnode.text().c_str());
        build_children = false;
      }
      else if (tag == "em") {
        node = cmark_node_new(CMARK_NODE_EMPH);
      }
      else if (tag == "strong") {
        node = cmark_node_new(CMARK_NODE_STRONG);
      }
      else if (tag == "a" or tag == "img") {
        std::string url;
        if (tag == "a"){
          node = cmark_node_new(CMARK_NODE_LINK);
          url = xnode.attr("href");
        }
        else {
          node = cmark_node_new(CMARK_NODE_IMAGE);
          url = xnode.attr("src");
        }
        if (url.length()) cmark_node_set_url(node, url.c_str());
        auto title = xnode.attr("title");
        if (title.length()) cmark_node_set_title(node, title.c_str());
      }
      else {
        if (Html::is_block_element(tag)) node = cmark_node_new(CMARK_NODE_HTML_BLOCK);
        else node = cmark_node_new(CMARK_NODE_HTML_INLINE);
        cmark_node_set_literal(node, xnode.dump().c_str());
        build_children = false;
      }

      if (build_children) {
        for (auto child : xnode.children()) build_cmark_tree(node, child);
      }
    }

    // Wrap the new node as required.
    // cmark does not allow arbitary node trees (e.g. an item node can not be append as
    // a child of a item parent). See cmark's `S_can_contain` function (in `node.c`) which
    // defines permissible parent-child relationships. Also, to know how to wrap a node
    // it can be useful to look at cmark's XML tree given some supplied markdown using
    // the CLI : `cmark --to xml trial.md`.
    cmark_node_type parent_type = cmark_node_get_type(parent);
    cmark_node_type node_type = cmark_node_get_type(node);
    bool node_inline = node_type >= CMARK_NODE_FIRST_INLINE and node_type <= CMARK_NODE_LAST_INLINE;
    // Reflect what is in cmark's `S_can_contain` function
    switch(parent_type) {

      case CMARK_NODE_DOCUMENT:
      case CMARK_NODE_BLOCK_QUOTE:
      case CMARK_NODE_ITEM:
      if (node_inline) {
        cmark_node* paragraph = cmark_node_new(CMARK_NODE_PARAGRAPH);
        cmark_node_append_child(paragraph, node);
        node = paragraph;
      }
      break;

      case CMARK_NODE_LIST:
      if (node_type != CMARK_NODE_ITEM) {
        if (node_inline) {
          cmark_node* paragraph = cmark_node_new(CMARK_NODE_PARAGRAPH);
          cmark_node_append_child(paragraph, node);
          node = paragraph;
        }
        cmark_node* item = cmark_node_new(CMARK_NODE_ITEM);
        cmark_node_append_child(item, node);
        node = item;   
      }
      break;

      case CMARK_NODE_PARAGRAPH:
      case CMARK_NODE_HEADING:
      case CMARK_NODE_EMPH:
      case CMARK_NODE_STRONG:
      case CMARK_NODE_LINK:
      case CMARK_NODE_IMAGE:
      if (not node_inline) {
        STENCILA_THROW(
          Exception,
          std::string("Can not append a block child to this parent.") +
          "\n  parent: " + cmark_node_get_type_string(parent) +
          "\n  child: " + cmark_node_get_type_string(node)
        )
      }
      break;

      default:
      break;
    }

    bool success = cmark_node_append_child(parent, node);
    if (not success) {
      STENCILA_THROW(
        Exception,
        std::string("Unable to append child node type to parent.") +
        "\n  parent: " + cmark_node_get_type_string(parent) +
        "\n  child: " + cmark_node_get_type_string(node)
      )
    }

  }

} // namespace


Document& Document::md(const std::string& md) {
  if (root_) cmark_node_free(root_);
  root_ = cmark_parse_document(md.c_str(), md.length(), CMARK_OPT_DEFAULT);
  return *this;
}

std::string Document::md(int width) const {
  return root_?wrap_render(cmark_render_commonmark(root_, CMARK_OPT_DEFAULT, width)):"";
}

std::string Document::html(void) const {
  return root_?wrap_render(cmark_render_html(root_, CMARK_OPT_DEFAULT)):"";
}

Xml::Document Document::html_doc(void) const {
  Xml::Document doc;
  build_html_tree(doc, root_);
  return doc;
}

Document& Document::html_doc(const Xml::Document& doc) {
  if (root_) cmark_node_free(root_);
  root_ = cmark_node_new(CMARK_NODE_DOCUMENT);
  build_cmark_tree(root_, doc);
  cmark_consolidate_text_nodes(root_);
  return *this;
}

std::string Document::latex(int width) const {
  return root_?wrap_render(cmark_render_latex(root_, CMARK_OPT_DEFAULT, width)):"";
}

std::string Document::man(int width) const {
  return root_?wrap_render(cmark_render_man(root_, CMARK_OPT_DEFAULT, width)):"";
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
