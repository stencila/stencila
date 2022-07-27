# RPNG

A theme for reproducible PNGs (rPNGs). This theme is used in Encoda when generating rPNGs.

## Notes

When testing this theme locally be sure to add `components=none` to the URL to ensure that Web Components are not loaded (as is the case when screenshotting via Chromium) e.g.

```sh

http://127.0.0.1:9000/fixtures/articles/interactive/distributions.md?theme=rpng&components=none
```

- This theme is used for parts of documents e.g. `MathBlocks`, `CodeFragments` in different contexts that the demo pages here. e.g. in a Google Doc

- In the future, if necessary, we _may_ have different RPNG themes for those different contexts.

- Currently the preview of this theme in the demo is broken because it pulls in the stencila Web Components. You can disable those temporarily by removing those by removing the two relevant `<script>` tags in `src/index.html`.
