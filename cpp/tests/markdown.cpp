#include <sstream>

#include <boost/test/unit_test.hpp>
#include <boost/filesystem.hpp>

#include <stencila/markdown.hpp>

BOOST_AUTO_TEST_SUITE(markdown_quick)

using Stencila::Markdown::Document;

BOOST_AUTO_TEST_CASE(dump) {
  Document doc("foo");

  BOOST_CHECK_EQUAL(doc.md(), "foo\n");
  BOOST_CHECK_EQUAL(doc.html(), "<p>foo</p>\n");
  BOOST_CHECK_EQUAL(doc.latex(), "foo\n");
  BOOST_CHECK_EQUAL(doc.man(), ".PP\nfoo\n");
}

BOOST_AUTO_TEST_CASE(html) {
  // Tests of how cmark does conversions to HTML
  Document doc;

  BOOST_CHECK_EQUAL(doc.md("Inline `code`.").html(), "<p>Inline <code>code</code>.</p>\n");
  BOOST_CHECK_EQUAL(doc.md("```\ncode block\n```").html(), "<pre><code>code block\n</code></pre>\n");
}

BOOST_AUTO_TEST_CASE(read_write) {
  using namespace boost::filesystem;

  Document doc;

  auto path = (temp_directory_path() / unique_path()).string();
  std::ofstream file(path);
  file << "foo\n";
  file.close();

  doc.read(path);

  auto read_ = [](const std::string& path){
    std::ifstream file(path);
    std::stringstream buffer;
    buffer << file.rdbuf();
    return buffer.str();
  };

  doc.write(path+".md");
  BOOST_CHECK_EQUAL(read_(path+".md"), "foo\n");

  doc.write(path+".html");
  BOOST_CHECK_EQUAL(read_(path+".html"), "<p>foo</p>\n");

  doc.write(path+".groff");
  BOOST_CHECK_EQUAL(read_(path+".groff"), ".PP\nfoo\n");
}

BOOST_AUTO_TEST_SUITE_END()
