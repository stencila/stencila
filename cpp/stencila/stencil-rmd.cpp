#include <stencila/debug.hpp>
#include <stencila/markdown.hpp>
#include <stencila/stencil.hpp>

namespace Stencila {

std::string Stencil::rmd(void) const {
  Xml::Document modified = *this;
  // Convert exec directives to code chunks
  for (auto exec : modified.filter("[data-exec]")) {
    Xml::Document temp;
    auto pre = temp.append("pre");
    auto code = pre.append("code");
    std::string info = "r ";
    // TODO reconstruct info string from exec settings
    // Execute exec(node);
    // if (exec.off) {
    //    info += " eval=FALSE,";
    // }
    // The extra holds stuff that the exec settings don't deal with
    // The StencilExecComponent needs to retain that.
    if (exec.has("data-extra")) {
      info += exec.attr("data-extra");
    }
    code.attr("class", "{" + info + "}");
    code.text(exec.text());
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
      if (info.length()>3) {
        std::string options = info.substr(3);
        if (options.back() == '}') options.pop_back();
        auto words = split(options, ",");
        for (auto word : words) {
          auto equal = word.find("=");
          if (equal != std::string::npos) {
            auto option = trim(word.substr(0,equal));
            auto value = trim(word.substr(equal+1));
            // eval:   whether to evaluate the chunk
            if (option == "eval" and (value == "FALSE" or value == "F")) {
              exec += " off";
            }
            // echo:   whether to include R source code in the output file
            else if (option == "echo" and (value == "TRUE" or value == "T")) {
              exec += " show";
            }
            // dev: the function name which will be used as a graphical device to record plots
            else if (option == "dev") {
              // value is usually a single or double quoted string literal so remove those
              value = value.substr(1,value.length()-2);
              exec += " format " + value;
            }
            // fig.width, fig.height: (both are 7; numeric) width and height of the plot, to be used in the graphics device (in inches) 
            // out.width, out.height: (NULL; character) width and height of the plot in the final output file (can be different with its real fig.width and fig.height, i.e. plots can be scaled in the output document)
            else if (option == "fig.width" or option == "out.width") {
              exec += " width " + value + "in";
            }
            else if (option == "fig.height" or option == "out.height") {
              exec += " height " + value + "in";
            }
          }
        }
      }
      // Remove the code block so struture is as expected for
      // stencil exec directives: pre[data-exec]
      // Store the options string for writing back to md
      auto pre = code.parent();
      pre.attr("data-exec", exec);
      pre.attr("data-rmd", info);
      pre.text(code.text());
      code.destroy();
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
