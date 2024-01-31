import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `File` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/file.md
 */
@customElement('stencila-file')
@withTwind()
export class File extends Entity {
  @property()
  name: string

  @property()
  path: string

  @property({ attribute: 'media-type' })
  mediaType?: string

  override render() {
    return this.renderAsTreeItem()
  }

  _renderAsTreeItem() {
    return html`<sl-tree-item>
      <sl-icon name="file"></sl-icon>
      ${this.name}
    </sl-tree-item>`
  }

  renderAsTreeItem() {
    const open = async () => {
      const response = await fetch('/~open/' + this.path)
      const doc = await response.json()

      //TODO: You'd want to do something like this to open the doc in a new tab...
      //const view = document.createElement('stencila-live-view')
      //view.setAttribute('doc', doc.id)
      //document.body.append(view)
      console.log('Opening doc', doc)

      // But for demo purpose let's open up a new browser tab
      window.open(window.origin + '/' + this.path, '_blank')
    }
    const delete_ = () => {
      this.directoryAction('delete', this.path)
    }
    const rename = (event: Event) => {
      const to = (event.target as HTMLInputElement).value
      this.directoryAction('rename', this.path, to)
    }

    return html`<div class="ml-3 mt-3">
      <p class="text-xl">📄 ${this.name} <button @click=${open}>👁️</button></p>
      <p class="text-sm">
        ✏️ <input type="text" size="80" value=${this.path} @change=${rename} />
      </p>
      <p class="text-sm">
        <button @click=${delete_}>🗑️</button>
      </p>
    </div>`
  }
}
