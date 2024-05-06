# Stencila VSCode Extension

## 🛠️ Develop

### Testing locally

In order to test the extension in VSCode, you will need to do the following:

```sh
npm install
npm run compile
```

Then press `F5` to run the "VSCode Extension" debug task,

### Writing walkthroughs

This extension has walkthroughs. The VSCode [contribution point](https://code.visualstudio.com/api/references/contribution-points#contributes.walkthroughs) for walkthroughs is an object in `package.json`. This is fine for simple walkthroughs but for more complex ones, including demos where you would like to include command links to perform actions it becomes tedious, error prone, and brittle.

Instead this extension has a [walkthrough/compile.js](walkthrough/compile.js) script which compiles the `contributes.walkthroughs` JSON object from YAML and Markdown files in sub-folders of [walkthroughs](walkthroughs).

The [walkthrough/compile.js](walkthrough/compile.js) script is run as part of `npm run compile`. You can also compile walkthroughs separately (you may want to use `watchexec` or similar to automatically run this when files in `walkthroughs` change):

```sh
npm run compile-walkthroughs
```

Each walkthrough is a subfolder of [walkthroughs](walkthroughs) and has a `main.yaml` with the `title` and `description`:

```yaml
title: The title of the demo
description: A description of the demo
```

Each step in the walkthrough is written as a Markdown file with a specific format:

* A YAML header with the `title` of the step.

* The `description` of the step which supports a limited subset of Markdown including __emphasis__, **bold**, and ``code`` (note specific syntax). Also don't forget emoji 🦄! Use the special ``type:X`` link to insert the source fragment (see below) with index X at the end of ``empty.smd``. Links on their own line are rendered as a button.

* One or more `source` sections which define the source to be inserted into the document when `type:X` links are clicked. Source sections are separated from the description and each other by three hyphens `---`.
