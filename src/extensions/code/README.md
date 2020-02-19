# Code nodes extension

Provides syntax highlighting for `CodeFragment` and `CodeBlock` nodes using [Prism](https://prismjs.com/). Will not style executable node types like `CodeExpression` and `CodeChunk` which are styled by the base Stencila Web Components.

## Notes

- Currently this addon renders syntax highlighting in the browser. In the future, we may take a similar approach as for MathJax and pre-render HTML in Encoda. See "Usage with Node" on https://prismjs.com/.

- Currently, syntax highlighting is only enabled for a limited number of languages. There is a https://prismjs.com/plugins/autoloader/ plugin but that would not be usable with are Parcel bundling approach. That is one reason in favor of doing pre-rending in Encoda - a reader would not have to load the Javascript to every language.

- There are many [plugins](https://prismjs.com/plugins) that may be appropriate to either add here, or as other addons e.g. `prism-line-numbers`.
