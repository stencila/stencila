import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { NodeType } from '../../../types'
import { getNodeIcon } from '../../../ui/icons/nodeIcons'
import { TWLitElement } from '../../../ui/twind'

import gutterMarkerColours from './colours'

@customElement('stencila-gutter-marker')
class StencilaGutterMarker extends TWLitElement {
  @property({ type: Boolean })
  isFirstLine: boolean = false

  @property({ type: Boolean })
  isLastLine: boolean = false

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

  render() {
    return html`
      <div
        class="relative flex h-full"
        style="width: ${this.defaultLineHeight}px;"
      >
        ${this.nodes.length > 1
          ? this.renderGutterLine(this.nodes[1], true)
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
      'rounded-[5px]',
    ])

    return html`
      ${this.renderGutterLine(node)}
      <div
        class=${styles}
        style="height: ${this
          .defaultLineHeight}px; background-color: ${colour};"
      >
        <sl-icon library="stencila" name=${getNodeIcon(node)}></sl-icon>
      </div>
    `
  }

  renderGutterLine(node: NodeType, isParentNode?: boolean) {
    const colour = gutterMarkerColours[node]
    const borderRadius = this.setBorderRadius(isParentNode)
    return html`
      <div class="h-full w-1/2">
        <div
          class="w-full h-full"
          style="background-color: ${colour}; border-radius: ${borderRadius};"
        ></div>
      </div>
    `
  }
}

export { StencilaGutterMarker }
