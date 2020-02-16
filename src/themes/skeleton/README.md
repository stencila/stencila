# Skeleton

A theme with lots of bones but no flesh. Designed to be used as a starting point for creating new themes, it only has empty stubs for each semantic selector e.g.

```css
:--Code {
}

:--CodeBlock {
}

:--CodeChunk {
}
```

To create a new theme using `skeleton`, simply duplicate this folder, rename it to your theme‘s name and run `npm run generate:themes`.

This theme does not depend upon anything in `shared`. When developing your new theme based on `skelton`, you may want to make use of `shared/fonts`, `shared/styles` and/or `shared/js` to avoid reimplementation. See the top level [`README.md`](../README.md) for more.
