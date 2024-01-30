import { BlockInfo, EditorView, gutter, GutterMarker } from '@codemirror/view'

import { MappingEntry } from '../../../clients/format'
import { SourceView } from '../../source'

import { StencilaGutterMarker } from './component'
import nodeGutterMarkers from './markers'

class NodeGutterMarker extends GutterMarker {
  /**
   * Array of the dom nodes at the start of the current line
   */
  nodes: MappingEntry[]

  /**
   *
   */
  line: BlockInfo

  isFirstLine: boolean
  isLastLine: boolean
  lineHeight: number

  constructor(nodes: MappingEntry[], line: BlockInfo, lineHeight: number) {
    super()
    this.nodes = nodes
    this.line = line
    this.lineHeight = lineHeight
    this.isFirstLine = this.checkFirstLine(nodes[0], line)
    this.isLastLine = this.checkLastLine(nodes[0], line)
  }

  private checkFirstLine = (node: MappingEntry, line: BlockInfo) => {
    return node.start === line.from
  }

  private checkLastLine = (node: MappingEntry, line: BlockInfo) => {
    return node.end > line.from && node.end < line.to
  }

  toDOM = (): Node => {
    const dom = document.createElement(
      'stencila-gutter-marker'
    ) as StencilaGutterMarker

    dom.isFirstLine = this.isFirstLine
    dom.isLastLine = this.isLastLine
    dom.nodes = this.nodes.map((node) => node.nodeType)

    return dom
  }
}

const statusGutter = (sourceView: SourceView) => [
  gutter({
    lineMarker: (view: EditorView, line: BlockInfo) => {
      // fetch nodes and filter out any node types that are not part of the
      // guttermarkers object
      const nodes = sourceView
        .getNodesAt(line.from)
        .filter((node) =>
          Object.keys(nodeGutterMarkers).includes(node.nodeType)
        )

      if (nodes.length > 0) {
        // useful debugging
        // console.log(
        //   'line:',
        //   view.state.doc.lineAt(line.from).number,
        //   'line start:',
        //   line.from,
        //   'line end: ',
        //   line.to,
        //   'nodes:',
        //   nodes
        // )
        const lineHeight = view.defaultLineHeight

        return new NodeGutterMarker(nodes, line, lineHeight)
      }
      return null
    },
    initialSpacer: () => null,
  }),
]

export { statusGutter, StencilaGutterMarker }
