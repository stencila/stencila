import { TagStyle } from '@codemirror/language'
import { Tag } from '@lezer/highlight'
import {
  BlockContext,
  LeafBlockParser,
  MarkdownConfig,
  LeafBlock,
  Element,
  InlineParser,
  InlineContext,
} from '@lezer/markdown'

import { getLeafEnd } from '../utilty'

// import { markdownLanguage } from '@codemirror/lang-markdown'
// import { Parser } from '@lezer'

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
  class: 'el',
}

const createMarkerEl = (
  cx: BlockContext | InlineContext,
  start: number,
  length: number
): Element => cx.elt(instructMark.name, start, start + length)

const createInstructTextEl = (
  cx: BlockContext | InlineContext,
  start: number,
  end: number
): Element => cx.elt(instructText.name, start, end)

/**
 * Utility fucntion to create the elements for the
 * Instruct block syntax
 */
const parseIntructBlock = (cx: BlockContext, leaf: LeafBlock): void => {
  const marker = createMarkerEl(cx, leaf.start, MARK_LENGTH)
  const instruction = createInstructTextEl(cx, marker.to + 1, getLeafEnd(leaf))
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
class InsertInstructBlockParser implements LeafBlockParser {
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
class EditInstructBlockParser implements LeafBlockParser {
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
 * `LeafBlockParser` for the closing an instruct block
 * which edits content within the `%%` delimiters.
 * eg: `%%`
 */
class CloseEditInstructParser implements LeafBlockParser {
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

const INLINE_MARK_1 = '{@@'
const INLINE_MARK_2 = '@@}'
const INLINE_MARK_3 = '{%%'
const INLINE_MARK_4 = '%>'
const INLINE_MARK_5 = '%%}'

/**
 *  `InlineParser` for parsing an inline insert instruction
 *  eg `{@@ create a sentence about frogs @@}`
 */
class InsertInstructInlineParser implements InlineParser {
  name = instructInlineInsert.name
  parse(cx: InlineContext, next: number, pos: number): number {
    const elements = []
    if (cx.slice(pos, pos + INLINE_MARK_1.length) === INLINE_MARK_1) {
      // create opening mark
      const openMark = createMarkerEl(cx, pos, INLINE_MARK_1.length)
      elements.push(openMark)

      const closeMarkIndex = cx
        .slice(pos + INLINE_MARK_1.length, pos + cx.text.length)
        .search(INLINE_MARK_2)

      let textEnd: number
      let endMark: Element | undefined

      // check for closing delim
      // use existance of closing delim to determine end of text element
      if (closeMarkIndex !== -1) {
        endMark = createMarkerEl(
          cx,
          openMark.to + closeMarkIndex,
          INLINE_MARK_2.length
        )
        textEnd = endMark.from
      } else {
        textEnd = pos + cx.text.length
      }

      // add instruct text
      elements.push(createInstructTextEl(cx, openMark.to, textEnd))

      // add the end mark element if it exists
      if (endMark) {
        elements.push(endMark)
      }

      return cx.addElement(
        cx.elt(
          instructInlineInsert.name,
          pos,
          pos + closeMarkIndex + INLINE_MARK_2.length,
          elements
        )
      )
    }
    return -1
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
        insertBlockMarker.test(leaf.content)
          ? new InsertInstructBlockParser()
          : null,
      endLeaf: (_, line) => !insertBlockMarker.test(line.text),
    },
    {
      name: instructBlockEdit.name,
      leaf: (_, leaf) =>
        editBlockStart.test(leaf.content)
          ? new EditInstructBlockParser()
          : null,
      endLeaf: (_, line) => !editBlockStart.test(line.text),
    },
    {
      name: instructBlockEditClose.name,
      leaf: (_, leaf) =>
        editBlockEnd.test(leaf.content) ? new CloseEditInstructParser() : null,
      endLeaf: (_, line) => !editBlockEnd.test(line.text),
    },
  ],
  parseInline: [new InsertInstructInlineParser()],
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
