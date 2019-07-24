import { InlineContent, BlockContent, CreativeWork } from '../types'

import { TypeMap } from './type-map'

export type InlineNodesWithType = Exclude<
  InlineContent,
  string | null | boolean | number
>

export const inlineContentTypes: TypeMap<InlineNodesWithType> = {
  Code: 'Code',
  CodeBlock: 'CodeBlock',
  CodeExpr: 'CodeExpr',
  Delete: 'Delete',
  Emphasis: 'Emphasis',
  ImageObject: 'ImageObject',
  Link: 'Link',
  Quote: 'Quote',
  Strong: 'Strong',
  Subscript: 'Subscript',
  Superscript: 'Superscript'
}

export const blockContentTypes: TypeMap<BlockContent> = {
  CodeBlock: 'CodeBlock',
  CodeChunk: 'CodeChunk',
  Heading: 'Heading',
  List: 'List',
  ListItem: 'ListItem',
  Paragraph: 'Paragraph',
  QuoteBlock: 'QuoteBlock',
  Table: 'Table',
  ThematicBreak: 'ThematicBreak'
}

export const creativeWorkTypes: TypeMap<CreativeWork> = {
  CreativeWork: 'CreativeWork',
  Article: 'Article',
  AudioObject: 'AudioObject',
  CodeChunk: 'CodeChunk',
  CodeExpr: 'CodeExpr',
  Collection: 'Collection',
  Datatable: 'Datatable',
  ImageObject: 'ImageObject',
  MediaObject: 'MediaObject',
  SoftwareApplication: 'SoftwareApplication',
  SoftwareSourceCode: 'SoftwareSourceCode',
  Table: 'Table',
  VideoObject: 'VideoObject'
}
