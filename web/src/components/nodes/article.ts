import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { isContentWriteable } from '../../mode'
import StencilaProseEditor from '../editors/prose-editor/prose-editor'
import { twSheet } from '../utils/css'
import StencilaEntity from './entity'

const { tw, sheet } = twSheet()

@customElement('stencila-article')
export default class StencilaArticle extends StencilaEntity {
  static styles = [sheet.target]

  /**
   * The article editor instantiated if whole article is writeable
   */
  editor?: StencilaProseEditor

  /**
   * Override to create an editor if necessary
   *
   * The `articleElem` is removed once the editor has parsed it to
   * avoid duplicate ids.
   */
  connectedCallback(): void {
    if (isContentWriteable()) {
      const articleElem = this.querySelector('article')!
      const viewElem = document.createElement('div')
      this.appendChild(viewElem)

      this.editor = new StencilaProseEditor('article', articleElem, viewElem)

      articleElem.remove()
    }
  }

  /**
   * Override to cleanup editor
   */
  disconnectedCallback(): void {
    this.editor?.destroy()
  }

  /**
   * Override to create a render root in the LightDOM so that styled
   * elements get styled with stylesheets adopted by the document
   */
  protected createRenderRoot(): Element | ShadowRoot {
    return this
  }

  protected render() {
    return html`<slot></slot>`
  }
}
