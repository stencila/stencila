import { PropertyValues, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'
import './helpers/node-authors'
import { nodeCardParentStyles, nodeCardStyles } from './helpers/node-card'

type CharacterStats = {
  words: number
  characters: number
  charactersExcludingSpaces: number
}

/**
 * Web component representing a Stencila Schema `Paragraph` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md
 */
@customElement('stencila-paragraph')
@withTwind()
export class Paragraph extends Entity {
  @state()
  protected characterStats: CharacterStats | undefined = undefined

  private renderViewInfo(displayContent: boolean = false) {
    const view = this.documentView()

    return html`
      <div class=${nodeCardParentStyles(view)}>
        <slot name="content" class=${displayContent ? 'hidden' : ''}></slot>
        <stencila-node-card type="Paragraph" class=${nodeCardStyles(view)}>
          <stencila-basic-node-field slot="body" icon-name="authors">
            <div slot="content" class="text-sm">
              Word count<br />
              <ul class="py-1 pl-3">
                <li class="text-xs">Words: ${this.characterStats?.words}</li>
                <li class="text-xs">
                  Characters: ${this.characterStats?.charactersExcludingSpaces}
                </li>
              </ul>
            </div>
          </stencila-basic-node-field>
          <stencila-node-authors type="Paragraph">
            <slot name="authors"></slot>
          </stencila-node-authors>
        </stencila-node-card>
      </div>
    `
  }

  override renderStaticView() {
    return this.renderViewInfo()
  }

  override renderDynamicView() {
    return this.renderViewInfo()
  }

  override renderVisualView() {
    return this.renderViewInfo()
  }

  override renderSourceView() {
    return this.renderViewInfo(true)
  }

  protected override updated(changedProperties: PropertyValues<this>): void {
    super.updated(changedProperties)

    if (this.characterStats !== undefined) {
      return
    }

    const slots = this.shadowRoot.querySelectorAll('slot')
    let words = 0
    let characters = 0
    let charactersExcludingSpaces = 0

    for (const slot of slots) {
      if (slot.name === 'authors') {
        continue
      }

      const elements = slot.assignedElements({ flatten: true })

      for (const el of elements) {
        const elementWords = el.textContent.trim().split(/\s+/)
        characters += el.textContent.length
        words += elementWords.length
        charactersExcludingSpaces += elementWords.join().length
      }
    }

    this.characterStats = {
      ...this.characterStats,
      characters: characters,
      words,
      charactersExcludingSpaces,
    }
  }
}
