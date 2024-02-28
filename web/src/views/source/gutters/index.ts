import { BlockInfo, EditorView, gutter, GutterMarker } from '@codemirror/view'
import { BlockTypeList, ExecutableTypeList } from '@stencila/types'

import { MappingEntry } from '../../../clients/format'
import { ObjectClient } from '../../../clients/object'
import { SourceView } from '../../source'

import { NodeGutterMarkerEl } from './nodeTypeGutter'
import { StatusGutterMarkerEl } from './statusGutter'

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
      'stencila-node-gutter-marker'
    ) as NodeGutterMarkerEl

    dom.isFirstLine = this.checkFirstLine(this.nodes[0], this.line)
    dom.isLastLine = this.checkLastLine(this.nodes[0], this.line)
    dom.isSingleLine = dom.isFirstLine && dom.isLastLine
    dom.nodes = this.nodes.map((node) => node.nodeType)
    dom.defaultLineHeight = this.defaultLineHeight
    dom.currentLineHeight = this.line.height

    return dom
  }
}

class StatusGutterMarker extends GutterMarker {
  defaultLineHeight: number

  objectClient: ObjectClient

  node: MappingEntry

  doc: string

  constructor(
    doc: string,
    node: MappingEntry,
    objectClient: ObjectClient,
    defaultLineHeight: number
  ) {
    super()
    this.doc = doc
    this.node = node
    this.objectClient = objectClient
    this.defaultLineHeight = defaultLineHeight
  }

  override toDOM(): Node {
    const dom = document.createElement(
      'stencila-status-gutter-marker'
    ) as StatusGutterMarkerEl

    dom.defaultLineHeight = this.defaultLineHeight
    dom.doc = this.doc
    dom.nodeId = this.node.nodeId

    return dom
  }
}

const execStatusGutter = (
  sourceView: SourceView,
  objectClient: ObjectClient
) => [
  gutter({
    lineMarker: (view: EditorView, line: BlockInfo) => {
      const blockNode = sourceView.getNodeAt(line.from)

      // find the inline executables within that line
      // const inlineExecutables = sourceView
      //   .getNodesBetween(line.from, line.to)
      //   .filter(
      //     (node) =>
      //       ExecutableTypeList.includes(node.nodeType) &&
      //       InlineTypeList.includes(node.nodeType)
      //   )

      if (blockNode && ExecutableTypeList.includes(blockNode.nodeType)) {
        // check this first line of a block node
        if (blockNode.start >= line.from && blockNode.start < line.to) {
          return new StatusGutterMarker(
            sourceView.doc,
            blockNode,
            objectClient,
            view.defaultLineHeight
          )
        }
      }

      return null
      // TODO check line for inline Executables
    },
    initialSpacer: () => null,
  }),
]

const nodeTypeGutter = (sourceView: SourceView) => [
  gutter({
    lineMarker: (view: EditorView, line: BlockInfo) => {
      // fetch nodes and filter out any node types that are not part of the
      // guttermarkers object
      // also checks some positional
      const nodes = sourceView
        .getNodesAt(line.from)
        .filter((node) => BlockTypeList.includes(node.nodeType))

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

export {
  nodeTypeGutter,
  execStatusGutter,
  NodeGutterMarkerEl,
  StatusGutterMarkerEl,
}
