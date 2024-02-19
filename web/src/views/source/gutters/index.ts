import { BlockInfo, EditorView, gutter, GutterMarker } from '@codemirror/view'
import type { NodeType } from '@stencila/types'

import { MappingEntry } from '../../../clients/format'
import { SourceView } from '../../source'

import { StencilaGutterMarker } from './component'

// TODO: This should be all `Block` node types and should be
// an array/function in @stencila/types
// See https://github.com/stencila/stencila/issues/2044
const gutterMarkerElements: readonly NodeType[] = [
  'Claim',
  'CodeBlock',
  'CodeChunk',
  'DeleteBlock',
  'Figure',
  'ForBlock',
  'Heading',
  'IfBlock',
  'InsertBlock',
  'InstructionBlock',
  'List',
  'MathBlock',
  'ModifyBlock',
  'Paragraph',
  'QuoteBlock',
  'ReplaceBlock',
  'Section',
  'StyledBlock',
  'Table',
  'ThematicBreak',
] as const

class NodeGutterMarker extends GutterMarker {
  /**
   * Array of the dom nodes at the start of the current line
   */
  nodes: MappingEntry[]

  /**
   * `BlockInfo` instance of the current line
   */
  line: BlockInfo

  /**
   * the default line height of the `EditorView` instance
   */
  defaultLineHeight: number

  constructor(
    nodes: MappingEntry[],
    line: BlockInfo,
    defaultLineHeight: number
  ) {
    super()
    this.nodes = nodes
    this.line = line
    this.defaultLineHeight = defaultLineHeight
  }

  private checkFirstLine = (node: MappingEntry, line: BlockInfo) => {
    return node.start === line.from
  }

  private checkLastLine = (node: MappingEntry, line: BlockInfo) => {
    return node.end > line.from && node.end <= line.to + 1
  }

  override toDOM = (): Node => {
    const dom = document.createElement(
      'stencila-gutter-marker'
    ) as StencilaGutterMarker

    dom.isFirstLine = this.checkFirstLine(this.nodes[0], this.line)
    dom.isLastLine = this.checkLastLine(this.nodes[0], this.line)
    dom.isSingleLine = dom.isFirstLine && dom.isLastLine
    dom.nodes = this.nodes.map((node) => node.nodeType)
    dom.defaultLineHeight = this.defaultLineHeight
    dom.currentLineHeight = this.line.height

    return dom
  }
}

const statusGutter = (sourceView: SourceView) => [
  gutter({
    lineMarker: (view: EditorView, line: BlockInfo) => {
      // fetch nodes and filter out any node types that are not part of the
      // guttermarkers object
      // also checks some positional
      const nodes = sourceView
        .getNodesAt(line.from)
        .filter((node) => gutterMarkerElements.includes(node.nodeType))

      if (nodes.length > 0) {
        // useful debugging log - logs out line info and nodes picked up
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
        return new NodeGutterMarker(nodes, line, view.defaultLineHeight)
      }
      return null
    },
    initialSpacer: () => null,
  }),
]

export { statusGutter, StencilaGutterMarker }
