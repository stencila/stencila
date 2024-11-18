import { SuggestionStatus } from '@stencila/types'
import { apply } from '@twind/core'
import { css, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/commands/suggestion-commands'
import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `SuggestionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md
 */
@customElement('stencila-suggestion-block')
@withTwind()
export class SuggestionBlock extends Entity {
  @property({ attribute: 'suggestion-status' })
  suggestionStatus?: SuggestionStatus

  @property({ attribute: 'execution-ended', type: Number })
  executionEnded?: number

  @property({ attribute: 'execution-duration', type: Number })
  executionDuration?: number

  @property()
  feedback?: string

  /**
   * Toggle the visibility of this suggestion so it can
   * not be seen or interacted with when inactive.
   * 
   * Needs to default to `true` so that the first suggestion is shown.
   */
  @state()
  public isActive: boolean = true

  static override styles = css`
    :host {
      flex: 0 0 100%;
      width: 100%;
    }
  `

  override render() {
    const styles = apply([
      'transition-opacity duration-300',
      this.isActive
        ? 'ease-out-quart opacity-1 pointer-events-auto'
        : 'ease-in-quart opacity-0 pointer-events-none',
    ])

    return html`<div class=${styles}>
      <slot name="content"></slot>
    </div>`
  }
}
