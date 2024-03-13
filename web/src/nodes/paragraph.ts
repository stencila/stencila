import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { CharacterStatsEntity } from './character-stats-entity'
import { nodeCardParentStyles, nodeCardStyles } from './helpers/node-card'

import '../ui/nodes/execution-info'
import '../ui/nodes/execution-icon'
import '../ui/nodes/execution-text'
import './helpers/node-authors'

/**
 * Web component representing a Stencila Schema `Paragraph` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md
 */
@customElement('stencila-paragraph')
@withTwind()
export class Paragraph extends CharacterStatsEntity {
  private renderViewInfo(hideContent: boolean = false) {
    const view = this.documentView()

    return html`
      <div class=${nodeCardParentStyles(view)} slot="body">
        <slot name="content" class=${hideContent ? 'hidden' : ''}></slot>
        <stencila-node-card type="Paragraph" class=${nodeCardStyles(view)}>
          <div slot="body">
            <stencila-ui-execution-info>
              <stencila-ui-execution-icon
                slot="icon"
                icon-name="add-file"
              ></stencila-ui-execution-icon>
              <div slot="content">
                <stencila-ui-execution-text text-size="sm"
                  >Content</stencila-ui-execution-text
                >
                <stencila-ui-execution-text text-size="xs"
                  >More content</stencila-ui-execution-text
                >
              </div>
            </stencila-ui-execution-info>

            <stencila-ui-execution-info>
              <stencila-ui-execution-icon
                slot="icon"
                icon-name="add-file"
              ></stencila-ui-execution-icon>
              <div slot="content">
                <stencila-ui-execution-text text-size="sm"
                  >Content</stencila-ui-execution-text
                >
                <stencila-ui-execution-text text-size="xs"
                  >More content</stencila-ui-execution-text
                >
              </div>
            </stencila-ui-execution-info>
          </div>
        </stencila-node-card>
        <stencila-node-authors type="Paragraph">
          <slot name="authors"></slot>
        </stencila-node-authors>
      </div>
    `
  }

  override renderStaticView() {
    return this.renderDynamicView()
  }

  override renderDynamicView() {
    const view = this.documentView()

    return html` <div class=${nodeCardParentStyles(view)} slot="body">
      <slot name="content"></slot>
      <stencila-node-card type="Paragraph" class=${nodeCardStyles(view)}>
        <div slot="body"></div>
      </stencila-node-card>
    </div>`
  }

  override renderVisualView() {
    return this.renderDynamicView()
  }

  override renderSourceView() {
    return this.renderViewInfo(true)
  }
}
