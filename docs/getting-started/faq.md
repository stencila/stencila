# Frequently Asked Questions

**Q** I already use RMarkdown/JupyterNotebook. How can Stencila be useful for me?<br/>
**A** Stencila allows you collaborate with colleagues who use other tools than RMarkdown and Jupyter Notebook,
without you having to give up your favourite tool. Stencila Coverter :sparkles: makes it possible to open documents in
various formats (`Rmd`, `ipynb` and so on) in Stencila. The conversion is completly lossless and the file can be saved back
in the original format :sparkles: . You can open the file in your tool of choice and see the changes your collaborator made.
 Stencila's interface provides text editing environment similar to popular text editors making it a low-entry barrier
 for collaborators unfamiliar with RMarkdown or Jupyter Notebook.

With Stencila you can have interactive code cells in different programming languages by enabling various execution contexts.
The data can be passed between the cells through the data interchange feature.
These two features combined with the lossless conversion between formats means that researchers with different skillsets
can easily collaborate on the same document using their preferred programming language.

<hr/>
**Q** I don't write any code for my research. Is Stencila for me? <br/>
**A** Yes, definitely Stencila is for you! Stencila allows you to write your papers just like
you would do it in a popular text editor (MS Word or similar) but you can save it directly in a format
commonly used by publishers JATS [Journal Article Tag Suite](https://en.wikipedia.org/wiki/Journal_Article_Tag_Suite)
giving you more control over the formatting of your manuscript.

Using Stencila you can easily save your work :sparkles: in formats compatible with other popular reproducible research tools
(such as Jupyter Notebook) creating more opportunities for collaboration.

<hr />
**Q** What are Stencila Spreadheets?<br/>
**A** Stencila Sheets provide a way towards working within an environment similar to spreadsheet software but supporting
reproducible approach by linking spreadsheet directly to the article allowing for capturing the analysis steps.

Stencila will tie together the data in the spreadsheet, the methods you used to process the data and the researchers
publication.
![Example ofStencila Spreadsheet](img/stencila-mini-spreadsheet.png)

Stencila Spreadsheets make it possible for [extending the spreadsheet functionality](computation/functions.md#add-new-functions) by registering functions written in other
programming languages :sparkles: . Using simple Stencila API researchers can wrap up functions written in R, Python and other
languages, register them and thus make them available through the Stencila Spreadsheet interface.


<hr />
**Q** I would like to try out Stencila but don't want to (or can't) install the whole suite on my machine. What can I do?<br/>
**A** You can use [Stencila Hub](https://github.com/stencila/hub) :sparkles: . The Hub hosts interactive Stencila documents with
different execution contexts :sparkles: enabling collaboration and previewing Stencila Sheets, Articles and Notebooks.


<hr />
**Q** I am trying to use R with Stencila Notebook / Spreadsheet but I am getting this error message `No peers able to provide: R Context` <br/>
**A** Error message `No peers able to provide....` means that there is no active execution context for the language you are trying to use. In this particular case, if
you want to use `R` within Stencila, you need to make sure you enabled the R execution context. See the [installation instructions](installation.md) for details.
