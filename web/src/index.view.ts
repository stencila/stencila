/**
 * View mode
 *
 * Just imports web client for live, updating, previews of the document.
 */

import { main } from './index'

import './index.view.css'

window.stencilaWebClient = {
  main,
  executableLanguages: {},
}
