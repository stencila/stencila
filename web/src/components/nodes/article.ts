import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { isContentWriteable } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaEntity from './entity'

const { tw, sheet } = twSheet()

@customElement('stencila-article')
export default class StencilaArticle extends StencilaEntity {
  static styles = [sheet.target]

  render() {
    const readOnly = !isContentWriteable()

    return readOnly
      ? html`<slot></slot>`
      : html`<stencila-prose-editor schema="article"
          ><slot></slot
        ></stencila-prose-editor>`
  }
}
