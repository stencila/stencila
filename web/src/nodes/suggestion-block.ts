import { NodeType, SuggestionStatus } from '@stencila/types'
import { css, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { closestGlobally } from '../utilities/closestGlobally'

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
   * Should a node card, possibly within a suggestion, be expanded?
   */
  public static shouldExpand(card: HTMLElement, nodeType: NodeType): boolean {
    const types: NodeType[] = [
      'CodeBlock',
      'CodeChunk',
      'Datatable',
      'Figure',
      'ForBlock',
      'IfBlock',
      'IncludeBlock',
      'InstructionBlock',
      'MathBlock',
      'RawBlock',
      'StyledBlock',
      'Table',
    ]

    return (
      types.includes(nodeType) &&
      closestGlobally(card, 'stencila-suggestion-block') !== null
    )
  }

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
      height: 100%;
      min-width: 45ch;
      max-width: 65ch;
      margin: 0 auto;
      padding: 5px;
    }
  `

  override render() {
    return html`
      <div>${this.suggestionStatus}</div>
      <slot name="authors"></slot>
      <slot name="content"></slot>
    `
  }
}
