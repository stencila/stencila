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

import { getLeafEnd, hasOpeningDelimitir } from '../utilty'

const instructBlockRe = /^%%\s/
const startContentRe = /^%>$/
const endContentRe = /^%%$/

const customTags = {
  instructionBase: Tag.define(),
  instructionMark: Tag.define(),
  instructionText: Tag.define(),
  insertMark: Tag.define(),
}

// Instruct `NodeSpecs`
const instructBlock = {
  name: 'InstructBlock',
  style: customTags.instructionBase,
}

const startContentBlock = {
  name: 'InstructContentStartBlock',
  style: customTags.instructionBase,
}

const endContentBlock = {
  name: 'InstructContentEndBlock',
  style: customTags.instructionBase,
}

const instructInline = { name: 'InstructInline' }

const instructInlineClose = { name: 'InstructInlineClose' }

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

const BLOCK_MARK_LENGTH = 2

/**
 * Utility fucntion to create the elements for the
 * Instruct block syntax
 */
const parseBlockMarker = (
  cx: BlockContext,
  leaf: LeafBlock,
  name: string
): void => {
  const marker = createMarkerEl(cx, leaf.start, BLOCK_MARK_LENGTH)

  const elements = [marker]
  if (name === instructBlock.name) {
    const instruction = createInstructTextEl(
      cx,
      marker.to + 1,
      getLeafEnd(leaf)
    )
    elements.push(instruction)
  }
  cx.addLeafElement(leaf, cx.elt(name, leaf.start, getLeafEnd(leaf), elements))
}

/**
 * `LeafBlockParser` for parsing an instruction block
 * eg: `%% 4x10 table`
 */
class InsertBlockParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    parseBlockMarker(cx, leaf, instructBlock.name)
    return true
  }
}

/**
 * `LeafBlockParser` for the opening content of an instruct block
 * which edits content within the `%>` `%%` delimiters.
 * eg: `%%`
 */
class StartContentParser implements LeafBlockParser {
  nextLine = () => false
  finish(cx: BlockContext, leaf: LeafBlock): boolean {
    parseBlockMarker(cx, leaf, startContentBlock.name)
    return true
  }
}

/**
 * `LeafBlockParser` for the closing an instruct block
 * which edits content within the `%>` `%%` delimiters.
 * eg: `%%`
 */
class EndContentParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    parseBlockMarker(cx, leaf, endContentBlock.name)
    return true
  }
}

const INLINE_MARK_OPEN = '{%%'
const INLINE_MARK_CONTENT = '%>'
const INLINE_MARK_CLOSE = '%%}'

const createInlineElements = (
  cx: InlineContext,
  pos: number
): { elements: Element[]; closingPos: number } => {
  const elements = []

  const openMarkEl = createMarkerEl(cx, pos, INLINE_MARK_OPEN.length)
  elements.push(openMarkEl)

  const contentMarkIndex = cx
    .slice(pos + INLINE_MARK_OPEN.length, pos + cx.text.length)
    .search(INLINE_MARK_CONTENT)
  const closeMarkIndex = cx
    .slice(pos + INLINE_MARK_OPEN.length, pos + cx.text.length)
    .search(INLINE_MARK_CLOSE)

  let endIndex = -1
  let endMarker = INLINE_MARK_CLOSE

  if (contentMarkIndex !== -1 && closeMarkIndex !== -1) {
    console.log('hello')
    if (contentMarkIndex < closeMarkIndex) {
      endIndex = contentMarkIndex
      endMarker = INLINE_MARK_CONTENT
    } else {
      endIndex = closeMarkIndex
    }
  } else if (contentMarkIndex !== -1) {
    endIndex = contentMarkIndex
    endMarker = INLINE_MARK_CONTENT
  } else if (closeMarkIndex !== -1) {
    endIndex = closeMarkIndex
  }

  let textEnd: number
  let closeMarkEl: Element | undefined

  // check for closing delim, use pos to determine end of text element
  if (endIndex !== -1) {
    console.log(endMarker, endIndex)
    closeMarkEl = createMarkerEl(cx, openMarkEl.to + endIndex, endMarker.length)
    textEnd = closeMarkEl.from
  } else {
    textEnd = pos + cx.text.length
  }

  // add instruct text
  elements.push(createInstructTextEl(cx, openMarkEl.to, textEnd))

  // add the end mark element if it exists
  if (closeMarkEl) {
    elements.push(closeMarkEl)
  }

  return {
    elements,
    closingPos: closeMarkEl ? closeMarkEl.to : pos + cx.text.length,
  }
}

/**
 * `InlineParser` for an inline instruction for editing or insertion.
 * eg: `{%%improve this%> ...%%}` || `{%% write a paragraph about birds %%}`
 */
class IntructInlineParser implements InlineParser {
  name = instructInline.name
  parse = (cx: InlineContext, _next: number, pos: number): number => {
    if (cx.slice(pos, pos + INLINE_MARK_OPEN.length) === INLINE_MARK_OPEN) {
      const { elements, closingPos } = createInlineElements(cx, pos)
      cx.addElement(cx.elt(instructInline.name, pos, closingPos, elements))
    }
    return -1
  }
}

/**
 * `InlineParser` for the closing delim of an inline edit instruction.
 * eg: `%%}`
 */
class CloseInlineParser implements InlineParser {
  name = instructInlineClose.name
  parse = (cx: InlineContext, _next: number, pos: number): number => {
    if (
      cx.slice(pos, pos + INLINE_MARK_CLOSE.length) === INLINE_MARK_CLOSE &&
      hasOpeningDelimitir(cx, pos, INLINE_MARK_OPEN, INLINE_MARK_CLOSE)
    ) {
      return cx.addElement(
        cx.elt(instructInlineClose.name, pos, pos + INLINE_MARK_CLOSE.length, [
          createMarkerEl(cx, pos, INLINE_MARK_CLOSE.length),
        ])
      )
    }
    return -1
  }
}

/**
 * `MarkDownConfig` for applying syntax highlights to:
 *  - instruct blocks - as leaf nodes
 *  - instruct inline
 *
 * This is added as a codemirror `Extension` for the markdown language
 */
const StencilaInstructSyntax: MarkdownConfig = {
  defineNodes: [
    instructBlock,
    startContentBlock,
    endContentBlock,
    instructInline,
    instructInlineClose,
    instructMark,
    instructText,
  ],
  parseBlock: [
    {
      name: instructBlock.name,
      leaf: (_, leaf) =>
        instructBlockRe.test(leaf.content) ? new InsertBlockParser() : null,
      endLeaf: (_, line) => !instructBlockRe.test(line.text),
    },
    {
      name: startContentBlock.name,
      leaf: (_, leaf) =>
        startContentRe.test(leaf.content) ? new StartContentParser() : null,
      endLeaf: (_, line) => !startContentRe.test(line.text),
    },
    {
      name: endContentBlock.name,
      leaf: (_, leaf) =>
        endContentRe.test(leaf.content) ? new EndContentParser() : null,
      endLeaf: (_, line) => !endContentRe.test(line.text),
    },
  ],
  parseInline: [new IntructInlineParser(), new CloseInlineParser()],
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
