## Stencila

Stencila is a platform for creating, collaborating on, and sharing data driven content. Content that is transparent and reproducible, like RMarkdown and Jupyter. Content that can be versioned and composed just like we do with open source software using tools like CRAN and NPM. And above all, content that is accessible to non-coders, like Google Docs and Microsoft Office.

### Features

Features                                                                    | State
--------------------------------------------------------------------------- | :------------:
Documents                                                                   | ✓
Spreadsheets                                                                | 0.26+
Presentations                                                               | 0.26+
**Static Content**                                                          |
Paragraph                                                                   | ✓
Heading                                                                     | ✓
Blockquote                                                                  | ✓
Image                                                                       | ✓
List                                                                        | ✓
Table                                                                       | ✓
Strong                                                                      | ✓
Emphasis                                                                    | ✓
Link                                                                        | ✓
Subscript                                                                   | ✓
Superscript                                                                 | ✓
Code                                                                        | ✓
Math (AsciiMath and Tex)                                                    | ✓
**Data-driven content**                                                     |
Mini Cell (Expression)                                                      | ✓
Internal Code Cell (`call`)                                                 | ✓
External Code Cell (`run`)                                                  | 0.26+
Number Input (range slider)                                                 | ✓
Select Input (name value pairs)                                             | ✓
**Execution Engines**                                                       |
Javascript                                                                  | ✓
Node.js                                                                     | 0.26+
R                                                                           | 0.26+
Python                                                                      | 0.26+
Julia                                                                       | 0.26+
Scala                                                                       | 0.26+
**Functions**                                                               |
Basic Statistics                                                            | 0.26+
Tabular data manipulation (filter, aggregate, ...)                          | 0.26+
**Plotting**                                                                |
Base charts (Scatterplot, Lines, Bars, Pies, ...)                           | 0.26+
Extended charts (time series, box plot, candle stick, ...)                  | 0.26+
**Supported Formats**                                                       |
HTML                                                                        | ✓
JATS                                                                        | 0.26+
Markdown                                                                    | 0.26+
RMarkdown                                                                   | 0.26+
Jupyter Notebook                                                            | 0.26+
**Apps**                                                                    |
Stencila Desktop (Windows, OSX, Linux)                                      | ✓
Stencila Hub (web platform for collaboration)                               | 1.0


### Development

```
git clone https://github.com/stencila/ui.git
cd ui
npm install
npm run start
```

Now you can access the different interfaces in the browser:

- [http://localhost:4000/examples/dashboard](http://localhost:4000/examples/dashboard)
- [http://localhost:4000/examples/document](http://localhost:4000/examples/document)

Most development tasks can be run directly using Javascript tooling (`npm`)

Task                                                    | Command               |
------------------------------------------------------- |-----------------------|
Install and setup dependencies                          | `npm install`         |
Run the development server                              | `npm start`           |
Clean and rebuild                                       | `npm run build`       |

### Discuss

We love feedback. Create a [new issue](https://github.com/stencila/ui/issues/new), add to [existing issues](https://github.com/stencila/ui/issues) or [chat](https://gitter.im/stencila/stencila) with members of the community.
