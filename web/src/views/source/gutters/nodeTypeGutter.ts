import '@shoelace-style/shoelace/dist/components/icon/icon'

import type { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { nodeBorderColour, nodeUi } from '../../../nodes/helpers/node-ui'
import { withTwind } from '../../../twind'

@customElement('stencila-node-gutter-marker')
@withTwind()
class NodeGutterMarkerEl extends LitElement {
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

  private getGutterLineWidth = (): number => this.defaultLineHeight / 4

  private setBorderRadius = (isParentNode?: boolean): string => {
    if (!isParentNode) {
      if (this.isLastLine && !this.isFirstLine) {
        return '0 0 5px 5px'
      }
    }
    return 'none'
  }

  protected override render() {
    const width =
      this.defaultLineHeight +
      this.getGutterLineWidth() * (this.nodes.length - 1)
    return html`
      <div class="relative flex h-full" style="width: ${width}px;">
        ${this.nodes.reverse().map((node, i, arr) => {
          if (i === arr.length - 1) {
            return this.isFirstLine
              ? this.renderIcon(node, i)
              : this.renderGutterLine(node, i)
          }
          return this.renderGutterLine(node, i)
        })}
      </div>
    `
  }

  renderIcon(node: NodeType, depth: number = 0) {
    const { iconLibrary, icon, borderColour: colour } = nodeUi(node)

    const offset = depth * this.getGutterLineWidth()
    const styles = apply([
      `absolute top-0 left-[${offset}px]`,
      'flex justify-center items-center',
      `w-[${this.defaultLineHeight}px] h-[${this.defaultLineHeight}.px]`,
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
        <sl-icon library=${iconLibrary} name=${icon}></sl-icon>
      </div>
      ${!this.isSingleLine || this.currentLineHeight > this.defaultLineHeight
        ? this.renderGutterLine(node, depth)
        : ''}
    `
  }

  renderGutterLine(node: NodeType, depth: number = 0) {
    const colour = nodeBorderColour(node)
    const isLastNode = depth === this.nodes.length - 1
    const borderRadius = this.setBorderRadius(!isLastNode)

    return html`
      <div
        class="h-full ${this.isFirstLine && isLastNode ? 'pt-2' : ''}"
        style="width: ${this.getGutterLineWidth()}px;"
      >
        <div
          class="w-full h-full border-r border-gray-wild-sand"
          style="background-color: ${colour}; border-radius: ${borderRadius};"
        ></div>
      </div>
    `
  }
}

export { NodeGutterMarkerEl }
