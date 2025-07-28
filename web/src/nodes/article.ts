import { provide } from '@lit/context'
import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'
import {
  DocumentHeadingsContext,
  documentHeadingsContext,
} from '../ui/document/context'
import '../ui/document/article-headings'
import '../ui/document/article-references'

import { Entity } from './entity'
import { HEADING_VISIBILITY_EVENT, HeadingVisibilityEvent } from './heading'

@customElement('stencila-article')
@withTwind()
export class Article extends Entity {
    @property()
  doi?: string

  @property({ type: Array })
  identifiers?: unknown[]

  @property({ attribute: 'date-published' })
  datePublished?: string

  @property({ attribute: 'date-accepted' })
  dateAccepted?: string

  @property({ attribute: 'date-created' })
  dateCreated?: string

  @property()
  repository?: string

  @property()
  path?: string

  @property()
  commit?: string

  /**
   * Toggle show/hide abstract
   *
   * Defaults to false, and then is toggled off/on by user. Currently, only
   * observed for non-root articles
   */
  @state()
  private showAbstract?: boolean = false


  public static shouldExpand(_card: HTMLElement, nodeType: NodeType): boolean {
    return (
      nodeType == 'Article'
    )
  }

  /**
   * Context provider for the visibility of the article's headings
   *
   * Maintains the ids of the headings that are currently
   * visible (in the viewport) to enable highlighting of links
   * to those headings in the `<stencila-ui-article-headings>` component.
   */
  @provide({ context: documentHeadingsContext })
  headingsContext: DocumentHeadingsContext = {}

  /**
   * Handle a change in the visibility of a heading by
   * updating the `headingsContext`
   *
   * Warning: it is necessary to replace the context for changes
   * to reactively propagate to consumers of the context
   */
  handleHeadingVisibility({
    detail: { id, position, isEnd },
  }: CustomEvent<HeadingVisibilityEvent>) {
    const visibility = this.headingsContext[id] ?? [-1, -1]
    visibility[isEnd ? 1 : 0] = position
    this.headingsContext = { ...this.headingsContext, [id]: visibility }
  }

  override connectedCallback(): void {
    super.connectedCallback()
    if (this.isRoot()) {
      this.addEventListener(
        HEADING_VISIBILITY_EVENT,
        this.handleHeadingVisibility.bind(this)
      )
    }
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this.isRoot()) {
      this.removeEventListener(
        HEADING_VISIBILITY_EVENT,
        this.handleHeadingVisibility.bind(this)
      )
    }
  }

  override render() {
    return this.isRoot() ? this.renderAsRoot() : this.renderCard()
  }

  renderAsRoot() {
    return html`
      <stencila-ui-article-headings>
        <slot name="headings"></slot>
      </stencila-ui-article-headings>

      <slot name="title"></slot>
      
      <slot name="abstract"></slot>
      
      <slot name="content"></slot>

      <stencila-ui-article-references>
        <slot name="references"></slot>
      </stencila-ui-article-references>
    `
  }

  override renderCard() {
    return html`
      <stencila-ui-block-on-demand
        type="Article"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
        no-content-padding
      >
        <div slot="body" class="p-3">
          <slot name="reference"></slot>
          
          <div class="flex items-center justify-between mt-2">
            ${this.renderShowHideContent()}
          </div>
        </div>

        <div
          slot="content"
          class="px-3 transition-[padding-top,padding-bottom] duration-500 ease-in-out ${this
            .showAbstract
            ? 'py-3'
            : 'py-0'}"
        >
          <stencila-ui-collapsible-animation
            class=${this.showAbstract ? 'opened' : ''}
          >
            <div class="text-sm">
              <slot name="abstract"></slot>
            </div>
          </stencila-ui-collapsible-animation>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

    private renderShowHideContent() {
    return html`<sl-tooltip
      content=${this.showAbstract
        ? 'Hide excerpt content'
        : 'Show excerpt content'}
    >
      <stencila-ui-icon-button
        class="text-sm"
        name=${this.showAbstract ? 'eyeSlash' : 'eye'}
        @click=${(e: Event) => {
          // Stop the click behavior of the card header parent element
          e.stopImmediatePropagation()
          this.showAbstract = !this.showAbstract
        }}
      ></stencila-ui-icon-button>
    </sl-tooltip>`
  }
}
