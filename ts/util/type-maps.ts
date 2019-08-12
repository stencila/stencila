import * as types from '../types'
import { TypeMap } from './type-map'

export type MarkTypes =
  | types.Delete
  | types.Emphasis
  | types.Quote
  | types.Strong
  | types.Subscript
  | types.Superscript

export const markTypes: TypeMap<MarkTypes> = {
  Delete: 'Delete',
  Emphasis: 'Emphasis',
  Quote: 'Quote',
  Strong: 'Strong',
  Subscript: 'Subscript',
  Superscript: 'Superscript'
}

export type InlineNodesWithType = Exclude<
  types.InlineContent,
  string | null | boolean | number
>

export const inlineContentTypes: TypeMap<InlineNodesWithType> = {
  Cite: 'Cite',
  CiteGroup: 'CiteGroup',
  CodeFragment: 'CodeFragment',
  CodeExpression: 'CodeExpression',
  Delete: 'Delete',
  Emphasis: 'Emphasis',
  ImageObject: 'ImageObject',
  Link: 'Link',
  Quote: 'Quote',
  Strong: 'Strong',
  Subscript: 'Subscript',
  Superscript: 'Superscript'
}

export const blockContentTypes: TypeMap<types.BlockContent> = {
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

export const creativeWorkTypes: TypeMap<types.CreativeWork> = {
  CreativeWork: 'CreativeWork',
  Article: 'Article',
  AudioObject: 'AudioObject',
  Collection: 'Collection',
  Datatable: 'Datatable',
  ImageObject: 'ImageObject',
  Figure: 'Figure',
  MediaObject: 'MediaObject',
  Periodical: 'Periodical',
  PublicationIssue: 'PublicationIssue',
  PublicationVolume: 'PublicationVolume',
  SoftwareApplication: 'SoftwareApplication',
  SoftwareSourceCode: 'SoftwareSourceCode',
  Table: 'Table',
  VideoObject: 'VideoObject'
}

export type CodeTypes =
  | types.CodeFragment
  | types.CodeExpression
  | types.CodeBlock
  | types.CodeChunk

export const codeTypes: TypeMap<CodeTypes> = {
  CodeFragment: 'CodeFragment',
  CodeExpression: 'CodeExpression',
  CodeBlock: 'CodeBlock',
  CodeChunk: 'CodeChunk'
}
