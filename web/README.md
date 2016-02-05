Web browser interfaces to Stencila components.

For development you can use stenci.la as a backend by providing your user API token.

```bash
STENCILA_HUB_TOKEN=mytoken node server.js https://stenci.la
```

When adding a new node type to a stencil you need to replicate the file structure in existing node directories (under `packages`) and then require those files into the right places:

- xxx/StencilXxx.js in web/stencil/model/Stencil.js
- xxx/StencilXxxHTMLConverter.js in web/stencil/model/StencilHTMLConverters.js
- xxx/StencilXxxComponent.js in web/stencil/StencilWriter.js
- xxx/_xxx.scss in web/stencil/_stencil-base.scss
