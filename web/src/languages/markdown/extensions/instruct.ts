import { TagStyle } from '@codemirror/language'
import { Tag } from '@lezer/highlight'
import {
  BlockContext,
  LeafBlockParser,
  BlockParser,
  MarkdownConfig,
  LeafBlock,
  Element,
} from '@lezer/markdown'

const instructMarker = /@@\s/
const editBlockDelimiter = /%%\s/
const inlineInstructEdit = /{%%{([\S\s]*?)%>([\S\s]*?)}%%}/
const inlineInstructInsert = /{@@[\S\s]@@}/

const customTags = {
  instructMark: Tag.define(),
  instructDelimiter: Tag.define(),
}

const instructBlock = { name: 'InstructBlock', block: true }
const instructInline = { name: 'InstructInline' }
const instructMark = { name: 'InstructMark', style: customTags.instructMark }
const instructDelimiter = {
  name: 'InstructDelimiter',
  style: customTags.instructDelimiter,
}

const InstructBlockParser: BlockParser = {
  name: 
}

const StencilaInstructSyntax: MarkdownConfig = {
  defineNodes: [instructBlock, instructInline, instructMark, instructDelimiter],
}

export { StencilaInstructSyntax }
