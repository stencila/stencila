# Starting with Stencila

 Stencila Documents are like Microsoft Word or Google Docs documents but built from the ground up to fit with reproducible workflows. They allow you to embed code, in various programming langauges, within the document so that you can rapidly update your tables and figures - and so that others can see how you produced them. Stencila Documents can be stored using plain text formats like Markdown that work well with version control systems like Git. They can also be exported to publishing formats such as HTML (and in the future JATS and PDF).

 Compared to other popular tools for reproducible research such as Jupyter Notebooks and RMarkdown, Stencila Documents aim to have an interface that is more similar to the the familiar interface of a word processor and allow you to produce polished final documents.



You can use Stencila for reproducible publishing from the very start. Here's how to do that.

Try out [Stencila Alpha](http://alpha.stenci.la/example.html?archive=kitchen-sink)

### Cells

By default, code cells in Stencila Documents use a built-in simple expression language called [Mini](languages/mini/README.md).
You can [extend Stencila](installation,md#execution-contexts) with other languages suchas R, Python and SQL. Stencila allows you to have executable code for more than one
language within the same document.

### Loading data

Tabular data is an essential part of data analysis, so there is a special type called a <code>table.</code> The easiest way to create a table is by using the <code>csv</code> function which parses a string of comma separate text (in the future you'll be able to embed or link to your data more concisely that this! :sparkles: )



If you are using **Stencila Desktop**, once you launch it, you should be able to see the dashboard screen:

![Stencila Dashboard](img/stencila-dashboard.png)

Select `Welcome to Stencila` notebook and you are can now start working in the document.

### Manipulating data
