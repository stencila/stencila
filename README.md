# `thema`

> 🎨 Semantic themes for use with Stencila [`encoda`](https://github.com/stencila/encoda).

- [Quick Start](#quick-start)
- [Available Themes](#available-themes)
- [Theme Structure](#theme-structure)
- [Development](#development)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Technical Notes](#technical-notes)

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

## Development

### Prerequisites

- [Node.js](https://nodejs.org/en/)
- [npm](https://www.npmjs.com)
- [git](https://git-scm.com)

### Installation

1. Clone this repository

```sh
git clone git@github.com:stencila/thema.git
```

2. Install dependencies

```sh
npm install
```

3. Run development server

```sh
npm run dev
```

## Technical Notes

We use [Parcel](https://parceljs.org) to compile this module. One of the
plugins we utilize to generate transportable and offline-viewable documents
is [`parcel-plugin-url-loader`](https://github.com/stencila/parcel-plugin-url-loader)
to base64 encode and inline binary assets found in the CSS. This has the
tradeoff that it leads to much larger page sizes, but the ability to generate
stylesheets without inlining assets is on the roadmap.
