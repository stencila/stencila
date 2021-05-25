# Fixtures

This folder contains some examples of content that can be opened using Stencila including [`articles`](articles) and [`projects`](projects). They are used in automated and manual testing.

## ✨ Updating

Be careful when changing files since that will probably break tests that rely upon them.

Run `make -C articles` to update the article fixtures.

## 🚀 Serving

You may want to serve these fixtures so that you can use them when developing components of Stencila (e.g. [`viewer`](../viewer)).

You can do that using the `stencila` CLI tool,

```sh
stencila serve
```

Or, you could use a specialized static file server e.g. the `serve` NPM package:

```sh
npx serve --cors
```
