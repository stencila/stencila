/**
 * Thema demo
 *
 * Provides an interface for switching of both example and theme.
 * Used for human 🤗 user acceptance testing 👍 and robot 🤖
 * visual regression testing.
 */

import React from 'react'
import ReactDOM from 'react-dom'
import { ThemeEditor } from './editor'

ReactDOM.render(<ThemeEditor />, document.getElementById('sidebar'))
