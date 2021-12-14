/**
 * Edit mode
 *
 * Brings in the web client and the article editor.
 */

import { main } from './index'
export { ArticleEditor } from './editors/article'

window.stencilaWebClient = {
  main,
  executableLanguages: {},
}
