import { consume } from '@lit/context'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import {
  DocumentHeadingsContext,
  documentHeadingsContext,
} from '../ui/document/context'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Link` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md
 */
@customElement('stencila-link')
export class Link extends Entity {
  @property()
  target: string

  /**
   * Whether this is a link, in a <nav> element, to a header
   */
  isHeaderLink: boolean = false

  /**
   * Whether the link is active
   *
   * Only applicable if `isHeaderLink`. Reflected as a property
   * so it can be used for styling.
   */
  @property({ attribute: 'active', type: Boolean, reflect: true })
  isActive: boolean = false

  /**
   * The context containing the list of visible headings
   *
   * The `isVisible` method of the context is on render
   * if this `isHeaderLink`.
   */
  @consume({ context: documentHeadingsContext, subscribe: true })
  @state()
  headingsContext: DocumentHeadingsContext

  override connectedCallback() {
    super.connectedCallback()

    // Determine this initially, rather than repeatedly in `update()`
    this.isHeaderLink = this.closestGlobally('nav[slot=headings]') !== null
  }

  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)

    if (this.isHeaderLink && changedProperties.has('headingsContext')) {
      const id = this.target?.slice(1)
      const visibility = this.headingsContext[id]
      // TODO: consider al alternative to disabling this lint error
      // eslint-disable-next-line lit/no-property-change-update
      this.isActive = visibility
        ? visibility[0] === 0 || // <stencila-heading> within viewport
          visibility[1] === 0 || // <stencila-heading-end> within viewport
          (visibility[0] > 0 && visibility[1] < 0) // <stencila-heading> above && <stencila-heading-end> below
        : false
    }
  }

  override render() {
    return html`<slot></slot>`
  }
}
