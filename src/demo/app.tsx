/**
 * Thema demo
 *
 * Provides an interface for switching of both example and theme.
 * Used for human 🤗 user acceptance testing 👍 and robot 🤖
 * visual regression testing.
 *
 * For HTML content and Javascript modules,
 * switching is achieved via Parcel's [dynamic importing of modules]
 * (https://parceljs.org/code_splitting.html). Note that this seems to load
 * content for all examples and all themes i.e. it is not truly lazy.
 *
 * For CSS, this `import()` approach did not work, maybe because it loads all the
 * CSS stylesheets into the global DOM. So, we take the approach of
 * enabling/disabling `<link>` elements.
 */

import React from 'react'
import ReactDOM from 'react-dom'
import { ThemeEditor } from './editor'

ReactDOM.render(<ThemeEditor />, document.getElementById('sidebar'))
