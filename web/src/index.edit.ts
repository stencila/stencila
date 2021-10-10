/**
 * Edit mode
 *
 * Brings in the web client and the article editor.
 */

import { main } from './index'
export { Article } from './components/article/edit'

window.stencilaWebClient = {
  main,
}
