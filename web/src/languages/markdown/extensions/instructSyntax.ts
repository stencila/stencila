import { TagStyle } from '@codemirror/language'
import { Tag } from '@lezer/highlight'
import {
  BlockContext,
  LeafBlockParser,
  MarkdownConfig,
  LeafBlock,
  Element,
} from '@lezer/markdown'

import { getLeafEnd } from '../utilty'

const MARK_LENGTH = 2

const insertBlockMarker = /^@@\s/
const editBlockStart = /^%%\s/
const editBlockEnd = /^%%$/
// const insertInline = /{@@([\S\s]*?)@@}/
// const editInline = /{%%{([\S\s]*?)%>([\S\s]*?)}%%}/

const customTags = {
  instructionBase: Tag.define(),
  instructionMark: Tag.define(),
  instructionText: Tag.define(),
}

const instructBlockInsert = {
  name: 'InstructBlockInsert',
  style: customTags.instructionBase,
}
const instructBlockEdit = {
  name: 'InstructBockEdit',
  style: customTags.instructionBase,
}
const instructBlockEditClose = {
  name: 'InstructBlockEditClose',
  style: customTags.instructionBase,
}
const instructInlineInsert = { name: 'InstructInsertInline' }
const instructInlineEdit = { name: 'InstructEditInline' }

const instructMark = { name: 'InstructMark', style: customTags.instructionMark }
const instructText = {
  name: 'InstructText',
  style: customTags.instructionText,
}

const createMarkerEl = (
  cx: BlockContext,
  start: number,
  length: number
): Element => cx.elt(instructMark.name, start, start + length)

const createTextEl = (cx: BlockContext, start: number, end: number): Element =>
  cx.elt(instructText.name, start, end)

/**
 * Utility fucntion to create the elements for the
 * Instruct block syntax
 */
const parseIntructBlock = (cx: BlockContext, leaf: LeafBlock): void => {
  const marker = createMarkerEl(cx, leaf.start, MARK_LENGTH)
  const instruction = createTextEl(cx, marker.to + 1, getLeafEnd(leaf))
  cx.addLeafElement(
    leaf,
    cx.elt(instructBlockInsert.name, leaf.start, getLeafEnd(leaf), [
      marker,
      instruction,
    ])
  )
}

/**
 * `LeafBlockParser` for parsing an instruction block
 * which inserts content.
 * eg: `@@ 4x10 table`
 */
class InsertBlockParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    try {
      parseIntructBlock(cx, leaf)
      return true
    } catch (_) {
      return false
    }
  }
}

/**
 * `LeafBlockParser` for an instruction block
 * which edits content within the `%%` delimiters.
 * eg: `%% write this paragraph in the style of Charles Dickens`
 */
class EditBlockParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    try {
      parseIntructBlock(cx, leaf)
      return true
    } catch (_) {
      return false
    }
  }
}

/**
 * `LeafBlockParser` for
 * which edits content within the `%%` delimiters.
 * eg: `@@ 4x10 table`
 */
class CloseEditParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    const marker = createMarkerEl(cx, leaf.start, MARK_LENGTH)

    cx.addLeafElement(
      leaf,
      cx.elt(instructBlockEditClose.name, leaf.start, getLeafEnd(leaf), [
        marker,
      ])
    )

    return true
  }
}

const StencilaInstructSyntax: MarkdownConfig = {
  defineNodes: [
    instructBlockInsert,
    instructBlockEdit,
    instructBlockEditClose,
    instructInlineInsert,
    instructInlineEdit,
    instructMark,
    instructText,
  ],
  parseBlock: [
    {
      name: instructBlockInsert.name,
      leaf: (_, leaf) =>
        insertBlockMarker.test(leaf.content) ? new InsertBlockParser() : null,
      endLeaf: (_, line) => !insertBlockMarker.test(line.text),
    },
    {
      name: instructBlockEdit.name,
      leaf: (_, leaf) =>
        editBlockStart.test(leaf.content) ? new EditBlockParser() : null,
      endLeaf: (_, line) => !editBlockStart.test(line.text),
    },
    {
      name: instructBlockEditClose.name,
      leaf: (_, leaf) =>
        editBlockEnd.test(leaf.content) ? new CloseEditParser() : null,
      endLeaf: (_, line) => !editBlockEnd.test(line.text),
    },
  ],
}

const INSTRUCT_SYNTAX_BG = 'rgba(0,255,0,0.1)'

const highlightStyles: TagStyle[] = [
  {
    tag: customTags.instructionBase,
    backgroundColor: INSTRUCT_SYNTAX_BG,
  },
  {
    tag: customTags.instructionMark,
    color: 'purple',
    fontWeight: 500,
    backgroundColor: INSTRUCT_SYNTAX_BG,
  },
  {
    tag: customTags.instructionText,
    color: '#140D5A',
    backgroundColor: INSTRUCT_SYNTAX_BG,
  },
]

export { StencilaInstructSyntax, highlightStyles }
