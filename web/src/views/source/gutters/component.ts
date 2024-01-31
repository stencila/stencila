import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { NodeType } from '../../../types'
import { getNodeIcon } from '../../../ui/icons/nodeIcons'

import gutterMarkerColours from './colours'

@customElement('stencila-gutter-marker')
@withTwind()
class StencilaGutterMarker extends LitElement {
  @property({ type: Boolean })
  isFirstLine: boolean

  @property({ type: Boolean })
  isLastLine: boolean

  @property({ type: Boolean })
  isSingleLine: boolean

  @property({ type: Array })
  nodes: NodeType[]

  /**
   * the default line height of the `EditorView` instance
   */
  @property({ type: Number })
  defaultLineHeight: number

  /**
   * the actual height of the line.
   * _will be the same as the default, unless line wrapping
   * or other extensions cause the line to be bigger_
   */
  @property({ type: Number })
  currentLineHeight: number

  private setBorderRadius = (isParentNode?: boolean): string => {
    if (!isParentNode) {
      if (this.isLastLine && !this.isFirstLine) {
        return '0 0 5px 5px'
      } else if (this.isFirstLine) {
        return '5px 5px 0 0'
      }
    }
    return 'none'
  }

  protected override render() {
    return html`
      <div
        class="relative flex h-full"
        style="width: ${this.defaultLineHeight}px;"
      >
        ${this.nodes.length > 1
          ? this.nodes
              .slice(1)
              .map((node, i) => this.renderGutterLine(node, i + 1))
          : ''}
        ${this.isFirstLine
          ? this.renderIcon(this.nodes[0])
          : this.renderGutterLine(this.nodes[0])}
      </div>
    `
  }

  renderIcon(node: NodeType) {
    const colour = gutterMarkerColours[node]

    const styles = apply([
      'absolute top-0 left-0',
      'flex justify-center items-center',
      'w-full',
      'p-1',
      'z-30',
      'rounded-[4px]',
    ])

    return html`
      <div
        class=${styles}
        style="height: ${this
          .defaultLineHeight}px; background-color: ${colour};"
      >
        <sl-icon library="stencila" name=${getNodeIcon(node)}></sl-icon>
      </div>
      ${!this.isSingleLine || this.currentLineHeight > this.defaultLineHeight
        ? this.renderGutterLine(node)
        : ''}
    `
  }

  renderGutterLine(node: NodeType, depth: number = 0) {
    const colour = gutterMarkerColours[node]
    const borderRadius = this.setBorderRadius(depth > 0)

    return html`
      <div
        class="h-full w-1/4 ${this.isFirstLine && depth === 0 ? 'pt-2' : ''}"
      >
        <div
          class="w-full h-full border-r border-gray-wild-sand"
          style="background-color: ${colour}; border-radius: ${borderRadius};"
        ></div>
      </div>
    `
  }
}

export { StencilaGutterMarker }
