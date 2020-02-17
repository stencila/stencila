# Thema

> 🎨 Semantic themes for use with Stencila [`encoda`](https://github.com/stencila/encoda).

[![Build Status](https://travis-ci.org/stencila/thema.svg?branch=master)](https://travis-ci.org/stencila/thema)
[![Visual Regression Tests](https://img.shields.io/badge/Argos%20CI-Visual%20Regression%20tests-informational?style=flat)](https://www.argos-ci.com/stencila/thema/builds)

- [Quick Start](#quick-start)
- [Available Themes](#available-themes)
- [Theme Structure](#theme-structure)
- [Develop](#develop)
  - [Prerequisites](#prerequisites)
  - [Getting started](#getting-started)
  - [Creating a new theme](#creating-a-new-theme)
    - [Scripted creation](#scripted-creation)
    - [Manual creation](#manual-creation)
    - [Approaches](#approaches)
  - [Contributing to `shared` styles and scripts](#contributing-to-shared-styles-and-scripts)
  - [Generated code](#generated-code)
  - [Testing](#testing)
  - [Notes](#notes)
- [Acknowledgments](#acknowledgments)

## Quick Start

```sh
npm install @stencila/thema
```

```css
/* myStyleSheet.css */
@import '@stencila/thema/dist/stencila/styles.css';
```

```js
/* myJavaScript.js */
@import '@stencila/thema/dist/stencila';
```

## Available Themes

- `stencila`: A custom designed theme suited for long-form reading,
  specifically research papers. It strives to deliver an optimal reading
  experience whilst giving figures and other media objects as much room as
  possible.
- `eLife`: Inspired by the eLife website, this theme provides as
  close of an approximation as possible for solely semantically structured
  documents such as Markdown.

## Theme Structure

There are two primary files inside each theme folder. The naming of these two
files is important, and they must not be changed since they are referred to
from `encoda`.

- `styles.css`: CSS and visual styles specific to the theme. We use PostCSS
  to compile the CSS, this is done to utilize PostCSS utilities such as
  autoprefixing vendor flags to selectors, and writing nested selectors.
- `index.ts`: Written in TypeScript, this file is loaded asynchronously. It is
  used to progressively enhance the theme with things like syntax highlighting
  of code blocks.

## Develop

### Prerequisites

- [`node`](https://nodejs.org/en/)
- [`npm`](https://www.npmjs.com)
- [`git`](https://git-scm.com)

### Getting started

The best way to get started is to develop CSS and JS for a theme with the live updating demo running.

Clone this repository,

```sh
git clone git@github.com:stencila/thema.git
cd thema
```

Install dependencies,

```sh
npm install
```

Run the development server,

```sh
npm run dev
```

Then open http://localhost:1234/index.html in your browser to view the demo.

### Creating a new theme

#### Scripted creation

The easiest way to create a new theme is:

```bash
npm run create:theme -- mytheme
```

Theme names should be all lower case, and start with a letter. This creates a new folder in `src/themes` and the following files:

- a `README.md` providing a description of the theme and notes for contributors,
- a `styles.css` file for the theme's CSS,
- a `index.ts` for any Typescript that the style may need

#### Manual creation

You can create this folder structure for your theme manually. If you prefer to use Javascript instead of Typescript, use a `index.js` file instead of `index.ts`. Then update the list of themes in `themes/themes.ts` and elsewhere using:

```bash
npm run generate:themes
```

#### Approaches

There are three broad approaches to developing a new theme, each epitomized in three of the themes in this repository:

- the [`skeleton`](./src/themes/skeleton/README.md) approach: flesh things out yourself; start from scratch with nothing but be unaffected from changes to `shared` styles and scripts

- the [`zombie`](./src/themes/zombie/README.md) approach: leverage existing styles and scripts in `shared` but be affected by any changes to them

- the [`bootstrap`](./src/themes/bootstrap/README.md) approach: reuse existing stylesheets from elsewhere by mapping between Thema's semantic selectors and existing selectors in those stylesheets

It is important to note that the `skeleton`, `zombie` and `bootstrap` themes are extremes of each of the approaches - they apply their approach to _all_ document node types. Depending on your theme, the best approach is probably some combination of these approaches for different node types e.g. starting from scratch for some nodes and using `shared` styles for others.

There are a few key rules enforced by Stylelint:

- All slectors must be descendants of a custom semantic selector. This reduces risks of a theme interfering with
- exsitng stylesheets on a website.
- Avoid hard-coded values for things such as font-sizes, colors, and fonts. Instead, use CSS variables, as these will
- allow simple theme overrides within the browser without having to rebuild the theme.
- Design your themes using a mobile-first approach, adding overrides to refine styles on larger screens.
    At the same time, it is highly recommended to also include print media overrides with your theme.
- These themes are primarily intended for rendering interactive articles and other relatively long forms of prose.
    - As such, good typography is paramount. Reading Matthew Butterick‘s [Typography in Ten
    - Minutes](https://practicaltypography.com/typography-in-ten-minutes.html) will give you a solid foundation and a
    - reference for crafting your own themes.

### Contributing to `shared` styles and scripts

### Generated code

Some files in the `src` directory are auto-generated and should not be edited manually. Generated files include:

- `src/themes/themes.ts`: from the list of sub-folders in the `themes` folder
- `src/examples/examples.ts`: from the functions defined in `generate/examples.ts`
- `src/examples/*`: files generated by those functions, usually using the `@stencila/encoda` package
- `src/selectors.css`: custom selectors from the JSON Schema in the `@stencila/schema` package
- `src/shared/styles/mathjax.css`: MathJax CSS from the `mathjax-node` package

Run `npm run generate` when you add new themes, examples or upgrade one of those upstream packages. In particular, when Encoda is upgraded it is important to regenerate HTML for the examples; `npm run generate` is called after `npm install` to ensure this.

### Testing

We use visual regression testing powered by [Sauce Labs](https://saucelabs.com), [Argos](https://www.argos-ci.com), and [WebdriverIO](https://webdriver.io).

As part of the continuous integration for this repository, for each push,

1. themes are [built on Travis](https://travis-ci.org/stencila/thema),
2. examples rendered in a browser running on Sauce Labs,
3. screenshots taken and [uploaded to Argos](https://www.argos-ci.com/stencila/thema/builds).

To run these tests locally, run `npm run test`.
By default Webdriver will try to run the tests using Chrome, but you can switch to Firefox by setting an `TEST_BROWSER=firefox` environment variable.

When testing locally, there are three screenshot folders to be aware of inside the `test/screenshots` directory:

- `reference`: These are the baseline screenshots. To generate them, make sure your Git branch is free of any changes, and run `npm test`.
- `local`: These are screenshots generated by Webdriver tests, and will be compared to those found in the `reference` directory.
- `diff`: If any discrepancies are found between the `reference` and `local` screenshots, the differences will be highlighted and saved to this directory.

There is a pseudo-test in `test/screenshot.test.js` which can be un-skipped to help with debugging the automated running of tests.

### Notes

We use [Parcel](https://parceljs.org) to compile this module. One of the
plugins we utilize to generate transportable and offline-viewable documents
is [`parcel-plugin-url-loader`](https://github.com/stencila/parcel-plugin-url-loader)
to base64 encode and inline binary assets found in the CSS. This has the
tradeoff that it leads to much larger page sizes, but the ability to generate
stylesheets without inlining assets is on the roadmap.

## Acknowledgments

We rely on many tools and services for which we are grateful ❤ to their developers and contributors for all their time and energy.

|                                                 Tool                                                 | Use                                  |
| :--------------------------------------------------------------------------------------------------: | ------------------------------------ |
|    <a href="https://saucelabs.com"><img src="./.github/PoweredBySauceLabs.svg" width="150" /></a>    | Cross-browser testing platform       |
| <a href="https://webdriver.io/"><img src="https://webdriver.io/img/webdriverio.png" width="50"/></a> | WebDriver test framework for Node.js |
|        <a href="https://www.argos-ci.com/"><img src="./.github/ArgosCI.svg" width="150"/></a>        | Visual regression system             |
