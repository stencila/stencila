import { BlockInfo, EditorView, gutter, GutterMarker } from '@codemirror/view'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { MappingEntry } from '../../../clients/format'
import { NodeType } from '../../../types'
import { TWLitElement } from '../../../ui/twind'
import { SourceView } from '../../source'

import nodeGutterMarkers from './markers'

@customElement('stencila-gutter-marker')
class StencilaGutterMarker extends TWLitElement {
  @property({ type: Boolean })
  isFirstLine: boolean = false

  @property({ type: Boolean })
  isLastLine: boolean = false

  @property({ type: Array })
  nodesAtLine: NodeType[]

  render() {
    return html`
      <div class="w-4 h-4 relative">
        ${!this.isLastLine ? this.renderBase() : ''}
        ${this.isFirstLine ? this.renderIcon() : ''}
        ${this.isLastLine && !this.isFirstLine ? this.renderEnd() : ''}
      </div>
    `
  }

  renderIcon(zIndex?: number) {
    return html`
      <img
        src=${nodeGutterMarkers[this.nodesAtLine[0]].icon}
        class="absolute top-0 left-0"
        width="100%"
        height="100%"
        style="z-index=${zIndex ?? 10};"
      />
    `
  }

  renderBase() {
    return html`<div
      class="h-full w-1/2"
      style="background-color: ${nodeGutterMarkers[this.nodesAtLine[0]]
        .colour};"
    ></div>`
  }

  renderEnd() {
    return html`<div
      class="h-full w-1/2 rounded-[]"
      style="background-color: ${nodeGutterMarkers[this.nodesAtLine[0]]
        .colour}; border-radius: 0 0 25px 25px;"
    ></div>`
  }
}

class NodeGutterMarker extends GutterMarker {
  /**
   * Array of the dom nodes at the start of the current line
   */
  nodes: MappingEntry[]
  line: BlockInfo

  isFirstLine: boolean
  isLastLine: boolean

  constructor(nodes: MappingEntry[], line: BlockInfo) {
    super()
    this.nodes = nodes
    this.line = line
    this.isFirstLine = this.checkFirstLine(nodes[0], line)
    this.isLastLine = this.checkLastLine(nodes[0], line)
  }

  private checkFirstLine = (node: MappingEntry, line: BlockInfo) => {
    return node.start === line.from
  }

  private checkLastLine = (node: MappingEntry, line: BlockInfo) => {
    return node.start === line.from
  }

  toDOM = (): Node => {
    const dom = document.createElement('stencila-gutter-marker')

    dom.setAttribute('isFirstLine', String(this.isFirstLine))
    dom.setAttribute('isLastLine', String(this.isLastLine))
    dom.setAttribute(
      'nodesAtLine',
      JSON.stringify(this.nodes.map((node) => node.nodeType))
    )

    return dom
  }
}

const statusGutter = (sourceView: SourceView) => [
  gutter({
    lineMarker: (view: EditorView, line: BlockInfo) => {
      const nodes = sourceView
        .getNodesAt(line.from)
        .filter((node) => !['Text', 'Article'].includes(node.nodeType))

      // const node = sourceView.getNodeAt(line.from)

      if (nodes.length > 0) {
        console.log(
          'line:',
          view.state.doc.lineAt(line.from).number,

          'line start:',
          line.from,
          'nodes:',
          nodes
        )
        return new NodeGutterMarker(nodes, line)
      }
      return null
    },
    initialSpacer: () => null,
  }),
]

export { statusGutter, StencilaGutterMarker }
