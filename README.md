# Thema

> 🎨 Semantic themes for use with Stencila [`encoda`](https://github.com/stencila/encoda).

[![Build Status](https://travis-ci.org/stencila/thema.svg?branch=master)](https://travis-ci.org/stencila/thema)
[![Code coverage](https://codecov.io/gh/stencila/thema/branch/master/graph/badge.svg)](https://codecov.io/gh/stencila/thema)
[![Visual Regression Tests](https://img.shields.io/badge/Argos%20CI-Visual%20Regression%20tests-informational?style=flat)](https://www.argos-ci.com/stencila/thema/builds)

- [Quick Start](#quick-start)
- [Themes](#themes)
  - [Current themes](#current-themes)
  - [Structure of themes](#structure-of-themes)
- [Web Components](#web-components)
- [Extensions](#extensions)
  - [Current extensions](#current-extensions)
  - [Use an extension](#use-an-extension)
  - [Develop an extension](#develop-an-extension)
- [Develop](#develop)
  - [Prerequisites](#prerequisites)
  - [Getting started](#getting-started)
  - [Creating a new theme](#creating-a-new-theme)
    - [Scripted creation](#scripted-creation)
    - [Manual creation](#manual-creation)
    - [Approaches](#approaches)
- [Notes](#notes)
  - [Generated code](#generated-code)
  - [Testing](#testing)
    - [DOM traversal and manipulation](#dom-traversal-and-manipulation)
    - [Visual regressions](#visual-regressions)
- [Acknowledgments](#acknowledgments)
- [Utilities API](#utilities-api)
  - [Functions](#functions)
  - [ready(func)](#readyfunc)
  - [first([elem], selector) ⇒ <code>Element</code> \| <code>null</code>](#firstelem-selector-%e2%87%92-codeelementcode--codenullcode)
  - [select([elem], selector) ⇒ <code>Array.&lt;Element&gt;</code>](#selectelem-selector-%e2%87%92-codearrayltelementgtcode)
  - [create([spec], [attributes], ...children) ⇒ <code>Element</code>](#createspec-attributes-children-%e2%87%92-codeelementcode)
  - [tag(target, [value]) ⇒ <code>string</code> \| <code>Element</code>](#tagtarget-value-%e2%87%92-codestringcode--codeelementcode)
  - [attrs(target, [attributes]) ⇒ <code>object</code> \| <code>undefined</code>](#attrstarget-attributes-%e2%87%92-codeobjectcode--codeundefinedcode)
  - [attr(target, name, [value]) ⇒ <code>string</code> \| <code>null</code>](#attrtarget-name-value-%e2%87%92-codestringcode--codenullcode)
  - [text(target, [value]) ⇒ <code>string</code> \| <code>null</code> \| <code>undefined</code>](#texttarget-value-%e2%87%92-codestringcode--codenullcode--codeundefinedcode)
  - [append(target, ...elems)](#appendtarget-elems)
  - [prepend(target, ...elems)](#prependtarget-elems)
  - [before(target, ...elems)](#beforetarget-elems)
  - [after(target, ...elems)](#aftertarget-elems)
  - [replace(target, ...elems)](#replacetarget-elems)
  - [wrap(target, elem)](#wraptarget-elem)

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

## Themes

### Current themes

<!-- prettier-ignore-start -->
<!-- THEMES-START -->

| Name                            | Description                                                                                                                                                                                                                                                                                      |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [bootstrap](./themes/bootstrap) | A theme that pulls itself up using Twitter's [Bootstrap](https://getbootstrap.com/) toolkit.                                                                                                                                                                                                     |
| [elife](./themes/elife)         | A theme for the journal eLife.                                                                                                                                                                                                                                                                   |
| [galleria](./themes/galleria)   | A theme for galleries of `CreativeWork` nodes.                                                                                                                                                                                                                                                   |
| [nature](./themes/nature)       | A theme for the journal Nature.                                                                                                                                                                                                                                                                  |
| [plos](./themes/plos)           | A theme for the journal PLoS.                                                                                                                                                                                                                                                                    |
| [rpng](./themes/rpng)           | A theme for reproducible PNGs (rPNGs). This theme is used in Encoda when generating rPNGs.                                                                                                                                                                                                       |
| [skeleton](./themes/skeleton)   | A theme with lots of bones but no flesh. Designed to be used as a starting point for creating new themes, it tries to be as unopinionated as possible.                                                                                                                                           |
| [stencila](./themes/stencila)   | A theme reflecting Stencila's brand and [design system](https://github.com/stencila/designa). It is based on the Skeleton theme, and demonstrates how to customize a theme using CSS variables.                                                                                                  |
| [wilmore](./themes/wilmore)     | A theme well suited for consuming long-form manuscripts and prose. Named after Edmond Dantés' alias, [“Lord Wilmore: An Englishman, and the persona in which Dantès performs random acts of generosity.“](https://en.wikipedia.org/wiki/The_Count_of_Monte_Cristo#Edmond_Dantès_and_his_aliases) |

<!-- THEMES-END -->
<!-- prettier-ignore-end -->

### Structure of themes

There are two primary files inside each theme folder. The naming of these two
files is important, and they must not be changed since they are referred to
from `encoda`.

- `styles.css`: CSS and visual styles specific to the theme. We use PostCSS
  to compile the CSS, this is done to utilize PostCSS utilities such as
  autoprefixing vendor flags to selectors, and writing nested selectors.
- `index.ts`: Written in TypeScript, this file is loaded asynchronously. It is
  used to progressively enhance the theme with things like syntax highlighting
  of code blocks.

## Web Components

The [default set of Web Components](https://github.com/stencila/designa/tree/master/packages/components) to provide interactivity to document nodes.

Currently, the following node types have Web Components. Encoda will encode these nodes types as custom HTML elements that get hydrated into these components.

| Node type        | Custom element               |
| ---------------- | ---------------------------- |
| `CodeChunk`      | `<stencila-code-chunk>`      |
| `CodeExpression` | `<stencila-code-expression>` |

More components will be added over time. In the meantime, the "pseudo-components" in sibling folders to this one, provide styling for some other node types.

## Extensions

Extensions provide styling, and potentially interactivity, for node types that do not yet have corresponding web components. They are like fledgling web components, each with it's own CSS (and/or Javascript), that you can import into your own theme. Over time we expect extensions to be promoted to the Stencila components library, thereby obviating the need to import them explicitly.

### Current extensions

<!-- prettier-ignore-start -->
<!-- EXTS-START -->

| Name                          | Description                                                                                                                                                                                                                                 |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [cite](./themes/cite)         | Provides styling for in-text citations (i.e. `Cite` and `CiteGroup` nodes) and bibliographies (i.e. `CreativeWork` nodes in the `references` property of another `CreativeWork`).                                                           |
| [cite-apa](./themes/cite-apa) | Provides styling for in-text citations and bibliographies in accordance with the [American Psychological Association (APA) style](https://en.wikipedia.org/wiki/APA_style).                                                                 |
| [cite-mla](./themes/cite-mla) | Provides styling for in-text citations and bibliographies in accordance with the [Modern Language Association (MLA) style](https://style.mla.org/).                                                                                         |
| [code](./themes/code)         | Provides syntax highlighting for `CodeFragment` and `CodeBlock` nodes using [Prism](https://prismjs.com/). Will not style executable node types like `CodeExpression` and `CodeChunk` which are styled by the base Stencila Web Components. |
| [headings](./themes/headings) | A temporary extensions that changes the way that `Heading` nodes are represented. Ensures that there is only one `<h1>` tag (for the `title` property) and that `Heading` nodes are represented as `<h${depth+1}>`.                         |
| [math](./themes/math)         | Provides styling of math nodes using MathJax fonts and styles. Use this if there is any likely to be math content, i.e. `MathFragment` and/or `MathBlock` nodes, in documents that your theme targets.                                      |
| [pages](./themes/pages)       | Provides a [`@media print` CSS at-rule](https://developer.mozilla.org/en-US/docs/Web/CSS/@page) to modify properties when printing a document e.g. to PDF.                                                                                  |
| [person](./themes/person)     | Provides styling of `Person` nodes e.g the `authors` of an article, or authors for each `citation` in it's `references`.                                                                                                                    |

<!-- EXTS-END -->
<!-- prettier-ignore-end -->

### Use an extension

To add an extension to your theme, you simply have to import it's CSS and Javascript.

First import it's `styles.css` file into your theme's `styles.css` file e.g.

```css
@import '../../extensions/cite-apa/styles.css';
```

Then, if it has one, import it's `index.ts` file into your theme's `index.ts` file e.g.

```ts
import '../../extensions/cite-apa'
```

### Develop an extension

The first question to ask when developing a new extension is: should I? Extensions are best suited for styling / features that are:

- likely to be used in several themes, and
- needed in a hurry

If it's not likely to be used in more than one or two themes then it's probably not worth creating an extension. If it's not needed in a hurry, then it is probably better to put the effort into contributing a web component to `@stencila/components`. Having said that, if your more comfortable writing a simple extension here, to try out some ideas for something that may become a fully fledged web component, we are grateful for _any_ contributions.

The easiest way to create a new extension is using:

```bash
npm run create:extension -- myext
```

That will update the `src/extensions/index.ts` file with a new entry for your extension and create a new folder in the `src/extensions` folder with the necessary files:

```
src/extensions/myext
├── README.md
└── styles.css
```

You can create the folder and files yourself if you prefer. Just remember to run `npm run update:extensions` afterwards to add your extension to the index. See the other extensions for approaches to writing the CSS and Javascript / Typescript for your extension.

Some extensions perform manipulation of the DOM to make it more amenable to achieving a particular CSS styling e.g. adding a wrapping `<div>`s. For performance reasons these manipulations should be kept to a minimum. In some cases, it may be better to make the necessary changes to Encoda's HTML codec. In those cases the DOM manipulations in the extension should be commented as being temporary, and be [linked to an issue in Encoda](https://github.com/stencila/encoda/issues) to make those changes permanent.

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

Build auto-generated files necessary for theme functionality and development,

```sh
npm run bootstrap
```

Run the development server,

```sh
npm run dev
```

Then open http://localhost:1234/index.html in your browser to view the demo.

There are a few URL query parameters which can be used to control the UI of the theme preview.

| Parameter | Default Value        | Description                                                                                |
| :-------- | :------------------- | :----------------------------------------------------------------------------------------- |
| `theme`   | `stencila`           | Sets the active theme for the preview                                                      |
| `example` | `articleKitchenSink` | Sets the content for the preview                                                           |
| `ui`      | `true`               | When set to `false` hides the header and theme customization sidebar from the preview page |
| `header`  | `true`               | When set to `false` hides only the header from the preview page                            |
| `sidebar` | `true`               | When set to `false` hides only the theme customization sidebar from the preview page       |

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
npm run update:themes
```

#### Approaches

There are three broad approaches to developing a new theme, each epitomized in three of the themes in this repository:

- the [`skeleton`](./src/themes/skeleton/README.md) approach: flesh things out yourself; start from scratch with nothing but be unaffected from changes to `shared` styles and scripts

- the [`bootstrap`](./src/themes/bootstrap/README.md) approach: reuse existing stylesheets from elsewhere by mapping between Thema's semantic selectors and existing selectors in those stylesheets

It is important to note that the `skeleton` and `bootstrap` themes are extremes of each of the approaches - they apply their approach to _all_ document node types. Depending on your theme, the best approach is probably some combination of these approaches for different node types e.g. starting from scratch for some nodes and using `shared` styles for others.

There are a few key rules enforced by Stylelint:

- All selectors must be descendants of a custom semantic selector. This reduces risks of a theme interfering with exsitng
  stylesheets on a website.
- Avoid hard-coded values for things such as font-sizes, colors, and fonts. Instead, use CSS variables, as these will
  allow simple theme overrides within the browser without having to rebuild the theme.
- Design your themes using a mobile-first approach, adding overrides to refine styles on larger screens. At the same
  time, it is highly recommended to also include print media overrides with your theme.
- These themes are primarily intended for rendering interactive articles and other relatively long forms of prose.
  - As such, good typography is paramount. Reading Matthew Butterick‘s [Typography in Ten
  - Minutes](https://practicaltypography.com/typography-in-ten-minutes.html) will give you a solid foundation and a
  - reference for crafting your own themes.

To tweak or adjust an existing theme, you may override some common CSS variables found in the themes.
Please refer to the specific theme documentation for available variables.

Type selectors

For types defined in http://schema.org (e.g. `Article`), or extensions such as,
http://schema.stenci.la (e.g. `CodeChunk`), http://bioschemas.org (e.g. `Taxon`) etc.

Conventions:

- use the same upper camel case as in the schema the type is defined in
- use a `[itemtype=...]` selector if possible (i.e. if Encoda encodes it in HTML)

Property selectors

For properties of types defined in schemas. Note that
some of these select an entire container property e.g. `authors` and
selector for a class, and some select items in those properties
e.g. `author` and select for a `itemprop`.

Conventions:

- use the same lower camel case as in the schema the property is defined in
- use a `.class` selector for container properties
- use a `[itemprop=...]` selector for singular properties, or items of container properties

There are several additional selectors which are not found as the Stencila Schema definitions. These are:

| Selector            | Description                                                                                                                                                                                                                                                                                                                                                                                | Target                                                                                                                        |
| :------------------ | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------- |
| `:--root`           | Used in place of the [`:root` CSS pseudo selector](https://developer.mozilla.org/en-US/docs/Web/CSS/:root). It maps to the root element [generated by Encoda](https://github.com/stencila/encoda/commit/2859e329345b6d0b6f572d64f8bd8cacf3637fc4). This is done to avoid potential clashes with external stylesheets, and to ensure that Thema only styles semantically annotated content. | `[data-itemscope=root]`                                                                                                       |
| `:--CodeBlockTypes` | Block level code elements                                                                                                                                                                                                                                                                                                                                                                  | `:--CodeBlock`, `:--CodeChunk`                                                                                                |
| `:--CodeTypes`      | Inline level code elements                                                                                                                                                                                                                                                                                                                                                                 | `:--CodeBlock`, `:--CodeChunk`, `:--Code`, `:--CodeError`, `:--CodeExpression`, `:--CodeFragment`, `:--SoftwareSourceCode`    |
| `:--ListTypes`      | List elements, both ordered and unordered, as well as other lists such as author affiliations and article references                                                                                                                                                                                                                                                                       | `:--Article:--root > :--affiliations`, `:--Collection`, `:--List`, `:--references > ol`                                       |
| `:--MediaTypes`     | These are elements which usually benefit from taking up a wider screen area. Elements such as images, video elements, code blocks                                                                                                                                                                                                                                                          | `:--CodeBlock`, `:--CodeChunk`, `:--Datatable`, `:--Figure`, `:--ImageObject`, `:--MediaObject`, `:--Table`, `:--VideoObject` |

## Notes

- Theme authors should be able to override the styles of the web components as part of their theme.

### Generated code

Some files in the `src` directory are auto-generated and should not be edited manually. Generated files include:

- `src/themes/themes.ts`: from the list of sub-folders in the `themes` folder
- `src/examples/examples.ts`: from the functions defined in `generate/examples.ts`
- `src/examples/*`: files generated by those functions, usually using the `@stencila/encoda` package
- `src/selectors.css`: custom selectors from the JSON Schema in the `@stencila/schema` package

Run `npm run update` when you add new themes, addons, or examples, or upgrade one of those upstream packages. In particular, when Encoda is upgraded it is important to regenerate HTML for the examples - `npm run update` is called after `npm install` to ensure this.

### Testing

#### DOM traversal and manipulation

Authors of themes and extensions, and contributors to the utility functions are encouraged to add tests. We use [Jest](https://jestjs.io/) as a testing framework. Jest ships with [`jsdom`](https://github.com/jsdom/jsdom) which simulates a DOM environment in Node.js, thereby allowing testing as though running in the browser. See existing `*.test.ts` files for examples of how to do that.

#### Visual regressions

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

## Acknowledgments

We rely on many tools and services for which we are grateful ❤ to their developers and contributors for all their time and energy.

|                                                 Tool                                                 | Use                                  |
| :--------------------------------------------------------------------------------------------------: | ------------------------------------ |
|    <a href="https://saucelabs.com"><img src="./.github/PoweredBySauceLabs.svg" width="150" /></a>    | Cross-browser testing platform       |
| <a href="https://webdriver.io/"><img src="https://webdriver.io/img/webdriverio.png" width="50"/></a> | WebDriver test framework for Node.js |
|        <a href="https://www.argos-ci.com/"><img src="./.github/ArgosCI.svg" width="150"/></a>        | Visual regression system             |

## Utilities API

Several utility functions are provided in the [`util`](./src/util) module for traversing and manipulating the DOM. These may be useful for theme and extension authors when there is a need to modify the HTML structure of the document (e.g. adding additional purely presentational elements such as social sharing buttons). Caution should be taken to not overly modify the content. Only use these functions for things that are not possible with CSS alone.

<!-- prettier-ignore-start -->
<!-- API-START -->
### Functions

<dl>
<dt><a href="#ready">ready(func)</a></dt>
<dd><p>Register a function to be executed when the DOM is fully loaded.</p>
</dd>
<dt><a href="#first">first([elem], selector)</a> ⇒ <code>Element</code> | <code>null</code></dt>
<dd><p>Select the first element matching a CSS selector.</p>
</dd>
<dt><a href="#select">select([elem], selector)</a> ⇒ <code>Array.&lt;Element&gt;</code></dt>
<dd><p>Select all elements matching a CSS selector.</p>
</dd>
<dt><a href="#create">create([spec], [attributes], ...children)</a> ⇒ <code>Element</code></dt>
<dd><p>Create a new element.</p>
</dd>
<dt><a href="#tag">tag(target, [value])</a> ⇒ <code>string</code> | <code>Element</code></dt>
<dd><p>Get or set the tag name of an element.</p>
</dd>
<dt><a href="#attrs">attrs(target, [attributes])</a> ⇒ <code>object</code> | <code>undefined</code></dt>
<dd><p>Get or set the attributes of an element</p>
</dd>
<dt><a href="#attr">attr(target, name, [value])</a> ⇒ <code>string</code> | <code>null</code></dt>
<dd><p>Get or set the value of an attribute on an element.</p>
</dd>
<dt><a href="#text">text(target, [value])</a> ⇒ <code>string</code> | <code>null</code> | <code>undefined</code></dt>
<dd><p>Get or set the text content of an element.</p>
</dd>
<dt><a href="#append">append(target, ...elems)</a></dt>
<dd><p>Append new child elements to an element.</p>
</dd>
<dt><a href="#prepend">prepend(target, ...elems)</a></dt>
<dd><p>Prepend new child elements to an element.</p>
</dd>
<dt><a href="#before">before(target, ...elems)</a></dt>
<dd><p>Insert new elements before an element.</p>
</dd>
<dt><a href="#after">after(target, ...elems)</a></dt>
<dd><p>Insert new elements after an element.</p>
</dd>
<dt><a href="#replace">replace(target, ...elems)</a></dt>
<dd><p>Replace an element with a new element.</p>
</dd>
<dt><a href="#wrap">wrap(target, elem)</a></dt>
<dd><p>Wrap an element with a new element.</p>
</dd>
</dl>

<a name="ready"></a>

### ready(func)
Register a function to be executed when the DOM is fully loaded.

**Kind**: global function
**Detail**: Use this to wrap calls to the DOM selection and manipulation functions
to be sure that the DOM is ready before working on it.

| Param | Type | Description |
| --- | --- | --- |
| func | <code>function</code> | Function to register |

**Example**
```js
ready(() => {
  // Use other DOM manipulation functions here
})
```
<a name="first"></a>

### first([elem], selector) ⇒ <code>Element</code> \| <code>null</code>
Select the first element matching a CSS selector.

**Kind**: global function
**Returns**: <code>Element</code> \| <code>null</code> - An `Element` or `null` if no match
**Detail**: This function provides a short hand for `querySelector` but
also allowing for the use of semantic selectors.
You can use it for the whole document, or scoped to a particular element.

| Param | Type | Description |
| --- | --- | --- |
| [elem] | <code>Element</code> | The element to query (defaults to the `window.document`) |
| selector | <code>string</code> | The selector to match |

**Example** *(Select the first element from the document matching selector)*
```js

first(':--CodeChunk')
```
**Example** *(Select the first element within an element matching the selector)*
```js

first(elem, ':--author')
```
<a name="select"></a>

### select([elem], selector) ⇒ <code>Array.&lt;Element&gt;</code>
Select all elements matching a CSS selector.

**Kind**: global function
**Returns**: <code>Array.&lt;Element&gt;</code> - An array of elements
**Detail**: Provides a short hand for using `querySelectorAll` but
also allowing for the use of semantic selectors. You can use it for
the whole document, or scoped to a particular element.

| Param | Type | Description |
| --- | --- | --- |
| [elem] | <code>Element</code> | The element to query (defaults to the `window.document`) |
| selector | <code>string</code> | The selector to match |

**Example** *(Select all elements from the document matching selector)*
```js

select(':--CodeChunk')
```
**Example** *(Select all elements within an element matching the selector)*
```js

select(elem, ':--author')
```
<a name="create"></a>

### create([spec], [attributes], ...children) ⇒ <code>Element</code>
Create a new element.

**Kind**: global function
**Detail**: This function allows creation of new elements using either a
(a) HTML string (b) CSS selector like string, or (c) an `Element`.
CSS selectors are are convenient way to create elements with attributes,
particularly Microdata elements. They can be prone to syntax errors however.
Alternatively, the second argument can
be an object of attribute name:value pairs.

| Param | Type | Description |
| --- | --- | --- |
| [spec] | <code>string</code> \| <code>Element</code> | Specification of element to create. |
| [attributes] | <code>object</code> \| <code>undefined</code> \| <code>null</code> \| <code>boolean</code> \| <code>number</code> \| <code>string</code> \| <code>Element</code> | Attributes for the element. |
| ...children | <code>undefined</code> \| <code>null</code> \| <code>boolean</code> \| <code>number</code> \| <code>string</code> \| <code>Element</code> | Child nodes to to add as text content or elements. |

**Example** *(Create a &lt;figure&gt; with id, class and itemtype attributes)*
```js

create('figure #fig1 .fig :--Figure')
// <figure id="fig1" class="fig" itemscope="" itemtype="http://schema.stenci.la/Figure">
// </figure>
```
**Example** *(As above but using an object to specify attributes)*
```js

create('figure', {
  id: 'fig1',
  class: 'fig',
  itemscope: '',
  itemtype: translate(':--Figure')
})
```
**Example** *(Create a Person with a name property)*
```js

create(':--Person', create('span :--name', 'John Doe'))
// <div itemscope="" itemtype="http://schema.org/Person">
//   <span itemprop="name">John Doe</span>
// </div>
```
<a name="tag"></a>

### tag(target, [value]) ⇒ <code>string</code> \| <code>Element</code>
Get or set the tag name of an element.

**Kind**: global function
**Returns**: <code>string</code> \| <code>Element</code> - The lowercase tag name when getting,
                            a new element when setting.
**Detail**: Caution must be used when setting the tag. This function
does not actually change the tag of the element (that is not possible)
but instead returns a new `Element` that is a clone of the original apart
from having the new tag name. Use the `replace` function where necessary
in association with this function.

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to get or set the tag |
| [value] | <code>string</code> | The value of the tag (when setting) |

**Example** *(Get the tag name as a lowercase string)*
```js

tag(elem) // "h3"
```
**Example** *(Setting the tag actually returns a new element)*
```js

tag(tag(elem, 'h2')) // "h2"
```
**Example** *(Change the tag name of an element)*
```js

replace(elem, tag(elem, 'h2'))
```
<a name="attrs"></a>

### attrs(target, [attributes]) ⇒ <code>object</code> \| <code>undefined</code>
Get or set the attributes of an element

**Kind**: global function
**Returns**: <code>object</code> \| <code>undefined</code> - The attributes of the element when getting, `undefined` when setting

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to get or set the attributes |
| [attributes] | <code>object</code> | The name/value pairs of the attributes |

<a name="attr"></a>

### attr(target, name, [value]) ⇒ <code>string</code> \| <code>null</code>
Get or set the value of an attribute on an element.

**Kind**: global function
**Returns**: <code>string</code> \| <code>null</code> - a `string` if the attribute exists, `null` if does not exist, or when setting

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to get or set the attribute |
| name | <code>string</code> | The name of the attribute |
| [value] | <code>string</code> | The value of the attribute (when setting) |

**Example** *(Set an attribute value)*
```js

attr(elem, "attr", "value")
```
**Example** *(Get an attribute)*
```js

attr(elem, "attr") // "value"
```
<a name="text"></a>

### text(target, [value]) ⇒ <code>string</code> \| <code>null</code> \| <code>undefined</code>
Get or set the text content of an element.

**Kind**: global function
**Returns**: <code>string</code> \| <code>null</code> \| <code>undefined</code> - `null` if there is no text content,
                                     `undefined` when setting

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to get or set the text content |
| [value] | <code>string</code> | The value of the text content (when setting) |

**Example** *(Set the text content)*
```js

text(elem, "text content")
```
**Example** *(Get the text content)*
```js

text(elem) // "text content"
```
<a name="append"></a>

### append(target, ...elems)
Append new child elements to an element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to append to |
| ...elems | <code>Element</code> | The elements to append |

<a name="prepend"></a>

### prepend(target, ...elems)
Prepend new child elements to an element.

**Kind**: global function
**Detail**: When called with multiple elements to prepend
will maintain the order of those elements (at the top
of the target element).

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to prepend to |
| ...elems | <code>Element</code> | The elements to prepend |

<a name="before"></a>

### before(target, ...elems)
Insert new elements before an element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element before which the elements are to be inserted |
| ...elems | <code>Element</code> | The elements to insert |

<a name="after"></a>

### after(target, ...elems)
Insert new elements after an element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element after which the elements are to be inserted |
| ...elems | <code>Element</code> | The elements to insert |

<a name="replace"></a>

### replace(target, ...elems)
Replace an element with a new element.

**Kind**: global function

| Param | Type | Description |
| --- | --- | --- |
| target | <code>Element</code> | The element to replace |
| ...elems | <code>Element</code> | The elements to replace it with |

<a name="wrap"></a>

### wrap(target, elem)
Wrap an element with a new element.

**Kind**: global function
**Detail**: This function can be useful if you need
to create a container element to more easily style
a type of element.

| Param | Description |
| --- | --- |
| target | The element to wrap |
| elem | The element to wrap it in |

**Example** *(Wrap all figure captions in a &lt;div&gt;)*
```js

select(':--Figure :--caption')
  .forEach(caption => wrap(caption, create('div')))
```

<!-- API-END -->
<!-- prettier-ignore-end -->
