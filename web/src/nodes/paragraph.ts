import { PropertyValues, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'
import './helpers/node-authors'
import { nodeCardParentStyles, nodeCardStyles } from './helpers/node-card'
import './widgets/node-detail-card'

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

    console.log(displayContent)

    return html`
      <div class=${nodeCardParentStyles(view)} slot="body">
        <slot name="content"></slot>
        <stencila-node-card type="Paragraph" class=${nodeCardStyles(view)}>
          <!-- TODO: This is the component to refactor -->
          <div
            class="flex flex-row items-start justify-start gap-x-2 w-full mb-4 text-black"
            slot="body"
          >
            <div class="pt-0.5">
              <sl-icon
                name="hash-number"
                library="stencila"
                class="text-base"
              ></sl-icon>
            </div>
            <div class="grow ml-3">
              <div class="grid grid-cols-5">
                <div class="flex flex-col col-span-4">
                  <span class="text-sm leading-tight col-span-2"
                    >${this.characterStats?.words ?? 0} words</span
                  >
                </div>
              </div>
            </div>
          </div>
          <!-- <stencila-ui-node-detail-card
            label="Writer"
            slot="body"
            title="Kusal Ekanayake"
            content="Dragonfly Data Science"
            colour="black"
          >
            <div slot="content">This is content</div>
            <div slot="sidebar">3 days ago</div>
          </stencila-ui-node-detail-card> -->
          <!-- <stencila-basic-node-field slot="body" icon-name="authors">
            <div slot="content">
              <div class="grid grid-cols-5">
                <div class="flex flex-col col-span-4">
                  <span class="text-xxs leading-tight">Writer</span>
                  <span class="text-sm leading-tight col-span-2"
                    >Kusal Ekanayake</span
                  >
                  <span class="text-xs leading-tight"
                    >Dragonfly Data Science</span
                  >
                </div>
                <div
                  class="col-span-1 text-[8px] leading-tight text-right flex justify-self-center"
                >
                  3 Days <br />ago
                </div>
              </div>
            </div>
          </stencila-basic-node-field> -->
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
