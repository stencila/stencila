# Testing

To test the extension in VSCode, you will need `cargo` and `node` installed. Ensure JavaScript dependencies are installed using,

```sh
npm install
```

Then, build the extension's JavaScript and `stencila` CLI binary (which contains the Stencila Language Server used by the extension using):

```sh
npm run compile
```

Finally, within VSCode, press `F5` to run the "VSCode Extension" debug task which will bring up an "Extension Development Host" with the Stencila extension loaded.

There is also a test suite which can be run using

```sh
npm test
```

If you prefer, there is also a `Makefile` with recipes for these and other tasks e.g.

```sh
make fix test
```

> [!NOTE] Not in root NPM workspace
>
> This package can not be part of the root NPM workspace in this
> repository as that causes issues with packaging (trust me, I tried :/)


# Writing walkthroughs

This extension has walkthroughs. The VSCode [contribution point](https://code.visualstudio.com/api/references/contribution-points#contributes.walkthroughs) for walkthroughs is an object in `package.json`. This is fine for simple walkthroughs but for more complex ones, including demos where you would like to include command links to perform actions it becomes tedious, error prone, and brittle.

Instead this extension has a [walkthroughs/compile.js](walkthroughs/compile.js) script which compiles the `contributes.walkthroughs` JSON object from YAML and Markdown files in sub-folders of [walkthroughs](walkthroughs).

The [walkthroughs/compile.js](walkthroughs/compile.js) script is run as part of `npm run compile`. You can also compile walkthroughs separately (you may want to use `watchexec` or similar to automatically run this when files in `walkthroughs` change):

```sh
npm run compile-walkthroughs
```

Each walkthrough is a subfolder of [walkthroughs](walkthroughs) and has a `main.yaml` with the `title` and `description`:

```yaml
title: The title of the demo
description: A description of the demo
```

Each step in the walkthrough is written as a Markdown file with a specific format:

- A YAML header with the `title` of the step

- The `description` of the step which supports a limited subset of Markdown including **emphasis**, **bold**, and `code` (note specific syntax). Links on their own line (including special command links) are rendered as a button. Also don't forget emoji 🦄!
