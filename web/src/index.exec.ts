/**
 * Exec mode
 *
 * Just imports web client for live, updating, previews of the document.
 * In the future may also bring in WebComponents for interaction with
 * `Parameter`, `CodeChunk` etc nodes
 */

import { main } from './index'

import './index.exec.css'

window.stencilaWebClient = {
  main,
}
