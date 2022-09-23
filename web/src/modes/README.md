# `modes`

This folder contains JavaScript bundles for each document mode. To enable the desired code splitting, each mode has a `mode.ts` file and a `mode.js` file.

The `.js` file is the Parcel entry for the mode. It asynchronously loads the transpiled `.ts` file at runtime. The `.ts` file has a dynamic `import()` for other `.ts` files. This results in the desired chain of dynamic imports at runtime.

For example, in write mode, `write.js` will be loaded which will load `write.977e58d4.js` (or whatever hash) which then loads `edit.5875c82a.js`, which then loads `inspect.8fdf31fb.js` etc etc.

Some modes do not import any JavaScript or CSS over and above their base: they are just modes which alter the behavior / permissions of previously loaded code (e.g. the `Develop` mode uses the same Web Components as the `Alter` mode but allows for merges back to the trunk document). In these cases, there is some unnecessary overhead (~2kb) of the async loading code that Parcel includes. It would be possible to avoid this by "skipping" the effectively empty `.ts` file for that mode. But for consistency, that is not done at this stage.

In addition to the above, Parcel will create Javascript files for all dynamic `import()`s in the code (e.g. for each CodeMirror language e.g. `python.4f9d534e.js`) and for shared code e.g. `dist.2cd2648d.js`.
