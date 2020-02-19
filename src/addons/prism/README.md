# Prism addon

Syntax highlighting for `CodeFragment` and `CodeBlock` nodes using [Prism](https://prismjs.com/). You can also use this addon to style executable node types like `CodeExpression` and `CodeChunk` if you do not use the `stencila-components` addon.

## Notes

- Currently this addon renders syntax highlighting in the browser. In the future, we may take a similar approach as for MathJax and pre-render HTML in Encoda. See "Usage with Node" on https://prismjs.com/.

- Currently, syntax highlighting is only enabled for a limited number of languages. There is a https://prismjs.com/plugins/autoloader/ plugin but that would not be usable with are Parcel bundling approach. That is one reason in favor of doing pre-rending in Encoda - a reader would not have to load the Javascript to every language.

- There are many [plugins](https://prismjs.com/plugins) that may be appropriate to either add here, or as other addons e.g. `prism-line-numbers`.
