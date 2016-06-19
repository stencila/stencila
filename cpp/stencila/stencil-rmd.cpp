#include <boost/regex.hpp>

#include <stencila/debug.hpp>
#include <stencila/markdown.hpp>
#include <stencila/stencil.hpp>
#include <stencila/string.hpp>

namespace Stencila {

std::string Stencil::rmd(void) const {
  Xml::Document modified = *this;
  // "Unwrap" any exec directives within  a `figure` element
  // and extract the `caption` or `figcaption` to put in
  // the `fig.cap` option and "unwrap"
  for (auto exec : modified.filter("figure>[data-exec]")) {
    auto figure = exec.parent();
    auto caption = figure.select("figcaption,caption");
    if (caption) {
      auto extra = exec.attr("data-extra");
      if (extra.length()) extra += ", ";
      extra += "fig.cap=\"" + caption.text() + "\"";
      exec.attr("data-extra", extra);
    }
    figure.after(exec);
    figure.destroy();
  }
  // Convert exec directives to code chunks
  for (auto exec : modified.filter("[data-exec]")) {
    Xml::Document temp;
    auto pre = temp.append("pre");
    auto code = pre.append("code");
    std::string info = "r";
    // Create an execute directive and use it to construct a set of
    // chunk options
    Execute dir;
    try {
      dir.parse(exec);
    }
    catch(const Exception& e) {
      code.attr("data-error", e.what());
    }

    std::vector<std::string> options;
    if (dir.off) {
      options.push_back("eval=FALSE");
    }
    if (dir.show) {
      options.push_back("echo=TRUE");
    }
    if (dir.format.expr.length()) {
      options.push_back("dev=\""+dir.format.expr+"\"");
    }
    if (dir.width.expr.length()) {
      static const boost::regex pattern("^(\\d+)(px|mm|cm|in)?$");
      boost::smatch matches;
      if (boost::regex_search(dir.width.expr, matches, pattern)) {
        // TODO convert other dimensions to inches as necessary
        options.push_back("fig.width=" + matches[1]);
      }
    }
    if (dir.height.expr.length()) {
      static const boost::regex pattern("^(\\d+)(px|mm|cm|in)?$");
      boost::smatch matches;
      if (boost::regex_search(dir.height.expr, matches, pattern)) {
        // TODO convert other dimensions to inches as necessary
        options.push_back("fig.height=" + matches[1]);
      }
    }
    if (exec.has("data-extra")) {
      options.push_back(exec.attr("data-extra"));
    }

    if (options.size()) {
      info += " " + join(options, ", ");
    }

    code.attr("class", "{" + info + "}");
    code.text(trim(exec.text()));
    exec.before(pre);
    exec.destroy();
  }
  // Ignore output (Rmd does not usually store output)
  for (auto out : modified.filter("[data-out]")) {
    out.destroy();
  }
  // Convert text directives to inline code
  for (auto text : modified.filter("[data-text]")) {
    Xml::Document temp;
    auto code = temp.append("code", "r " + text.attr("data-text"));
    text.before(code);
    text.destroy();
  }
  return Markdown::Document()
    .html_doc(modified)
    .md();
}

Stencil& Stencil::rmd(const std::string& rmd) {
  // Parse markdown, convert to a HTML document and set this stencil's content
  Markdown::Document md(rmd);
  static_cast<Xml::Document&>(*this) = md.html_doc();

  /*
  Find code blocks and convert to execute directives.
  This conversion, converts knitr chunk options to
  execute directive options. e.g

    {r chunklabel, cache=TRUE, eval=FALSE, dpi=100}

  Chunk labels and options are optional
   
  A full list of options is at http://yihui.name/knitr/options
  Some frequently used options are
    eval:   whether to evaluate the chunk
    results:   `'markup'`, `'asis'`, `'hold'`, `'hide'`
    tidy:   whether to reformat R code
    cache:   whether to cache results
    fig.width, fig.height, out.width, out.height:   device and output size of figures
    include:   whether to include the chunk results in output
    child:   filenames of child documents
    engine:   language name (R, python, ...)
   */
  for (auto code : filter("pre code[class]")) {
    auto info = code.attr("class");
    if (info == "{r}" or info.substr(0,3) == "{r ") {
      std::string exec = "r";
      std::vector<std::string> unhandled;
      bool figure = false;
      std::string figure_caption;
      if (info.length()>3) {
        std::string options = info.substr(3);
        if (options.back() == '}') options.pop_back();
        auto words = split(options, ",");
        for (auto word : words) {
          auto equal = word.find("=");
          if (equal != std::string::npos) {
            auto option = trim(word.substr(0,equal));
            auto value = trim(word.substr(equal+1));
            // Option valuses are R expressions
            // Do psuedo-evaluation of the value by stripping quotes around string expressions
            // Proper R evalution could be dne but then would require an R context
            // be available during this conversion.
            // TODO We could enable evaluation if a R context was attached and fallback to this
            // if not.
            if (value.length()) {
              char first = value.front();
              char last = value.back();
              if ((first == '"' and last == '"') or (first == '\'' and last == '\'') ) {
                value = value.substr(1,value.length()-2);
              }
            }

            // eval:   whether to evaluate the chunk
            if (option == "eval") {
              if (value == "FALSE" or value == "F") exec += " off";
            }
            // echo:   whether to include R source code in the output file
            else if (option == "echo") {
              if (value == "TRUE" or value == "T") exec += " show";
            }
            // dev: the function name which will be used as a graphical device to record plots
            else if (option == "dev") {
              figure = true;
              exec += " format " + value;
            }
            // fig.width, fig.height: (both are 7; numeric) width and height of the plot, to be used in the graphics device (in inches) 
            // out.width, out.height: (NULL; character) width and height of the plot in the final output file (can be different with its real fig.width and fig.height, i.e. plots can be scaled in the output document)
            else if (option == "fig.width" or option == "out.width") {
              figure = true;
              exec += " width " + value + "in";
            }
            else if (option == "fig.height" or option == "out.height") {
              figure = true;
              exec += " height " + value + "in";
            }
            else if (option == "fig.cap") {
              figure = true;
              figure_caption = value;
            }
            else {
              unhandled.push_back(trim(word));
            }
          }
        }
      }
      // Remove the code block so struture is as expected for
      // stencil exec directives: pre[data-exec]  
      auto pre = code.parent();
      pre.attr("data-exec", exec);
      pre.text(code.text());
      code.destroy();
      // Store the unhandled option strings so it can be used 
      // for writing back to rmd
      if (unhandled.size()) pre.attr("data-extra", join(unhandled,", "));
      // Wrap in a `figure` element and caption if appropriate
      if (figure) {
        auto figure = pre.wrap("figure");
        if (figure_caption.length()) {
          figure.prepend("figcaption", figure_caption);
        }
      }
    }
  }

  // Find inline code and convert to text directives
  for (auto code : filter("code")) {
    auto text = code.text();
    if (text.length()>2) {
        if (text.substr(0,2) == "r ") {
            Xml::Document temp;
            auto span = temp.append("span",{{"data-text", text.substr(2)}});
            code.before(span);
            code.destroy();
        }
    }
  }

  return *this;
}

} //namespace Stencila
