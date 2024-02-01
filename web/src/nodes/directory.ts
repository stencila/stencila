import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Directory` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/directory.md
 */
@customElement('stencila-directory')
@withTwind()
export class Directory extends Entity {
  @property()
  name: string

  @property()
  path: string

  override render() {
    return this.renderAsTreeItem()
  }

  renderAsTreeItem() {
    const newFile = (event: Event) => {
      const name = (event.target as HTMLInputElement).value
      this.directoryAction('create-file', this.path + '/' + name)
    }
    const newDir = (event: Event) => {
      const name = (event.target as HTMLInputElement).value
      this.directoryAction('create-directory', this.path + '/' + name)
    }
    const delete_ = () => {
      this.directoryAction('delete', this.path)
    }
    const rename = (event: Event) => {
      const to = (event.target as HTMLInputElement).value
      this.directoryAction('rename', this.path, to)
    }

    return html`<div class="ml-3 mt-3">
      <p class="text-xl">📁 ${this.name}</p>
      <p class="text-sm">
        ✏️ <input type="text" size="80" value=${this.path} @change=${rename} />
      </p>
      <p class="text-sm">
        ➕📄 <input type="text" placeholder="name" @change=${newFile} />
      </p>
      <p class="text-sm">
        ➕📁 <input type="text" placeholder="name" @change=${newDir} />
      </p>
      <p class="text-sm">
        <button @click=${delete_}>🗑️</button>
      </p>
      <slot name="parts"></slot>
    </div>`
  }
}
