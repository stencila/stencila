import { markdownLanguage } from '@codemirror/lang-markdown'
import { TagStyle } from '@codemirror/language'
import { parseMixed } from '@lezer/common'
import { Tag } from '@lezer/highlight'
import {
  InlineParser,
  InlineContext,
  MarkdownConfig,
  Line,
  BlockParser,
  BlockContext,
} from '@lezer/markdown'

const customTags = {
  insertBase: Tag.define(),
  insertMark: Tag.define(),
  modifyBase: Tag.define(),
  modifyMark: Tag.define(),
}

// TODO ->
// modify inline, modify block,
// replace block, replace inline,
// delete block, delete inline

// Insert --------------------------------------------

const insertBlockDelim = /^\+\+$/

// insert `NodeSpecs`
const insertBlock = {
  name: 'InsertBlock',
  block: true,
  style: customTags.insertBase,
}
const insertInline = { name: 'InsertInline', style: customTags.insertBase }
const insertBlockMark = {
  name: 'InsertBlockMark',
  style: customTags.insertMark,
}
const insertInlineMark = {
  name: 'InsertInlineMark',
  style: customTags.insertMark,
}

const BLOCK_MARK_LENGTH = 2
const INSERT_INLINE_OPEN = '{++'
const INSERT_INLINE_CLOSE = '++}'

const delimiter = { resolve: insertInline.name, mark: insertInlineMark.name }

/**
 * `BlockParser` for block insertions syntax highlighting
 * eg: `++`
 */
class InsertBlockParser implements BlockParser {
  name = insertBlock.name
  parse(cx: BlockContext, line: Line): boolean {
    if (!insertBlockDelim.test(line.text)) {
      return false
    }
    const start = cx.lineStart
    const elements = [
      cx.elt(
        insertBlockMark.name,
        cx.lineStart,
        cx.lineStart + BLOCK_MARK_LENGTH
      ),
    ]
    // iterate over lines until closing delim found
    while (cx.nextLine()) {
      if (insertBlockDelim.test(line.text)) {
        elements.push(
          cx.elt(
            insertBlockMark.name,
            cx.lineStart,
            cx.lineStart + BLOCK_MARK_LENGTH
          )
        )
        cx.addElement(
          cx.elt(insertBlock.name, start, cx.lineStart + 2, elements)
        )
        break
      }
    }
    return true
  }
}

/**
 * `InlineParser` for inline insertion syntax highlighting
 * eg: `{++ ++}`
 */
class InsertInlineParser implements InlineParser {
  name = insertInline.name
  parse = (cx: InlineContext, _next: number, pos: number): number => {
    // add open or close delim
    if (cx.slice(pos, pos + INSERT_INLINE_OPEN.length) === INSERT_INLINE_OPEN) {
      return cx.addDelimiter(
        delimiter,
        pos,
        pos + INSERT_INLINE_OPEN.length,
        true,
        false
      )
    } else if (
      cx.slice(pos, pos + INSERT_INLINE_CLOSE.length) === INSERT_INLINE_CLOSE
    ) {
      return cx.addDelimiter(
        delimiter,
        pos,
        pos + INSERT_INLINE_CLOSE.length,
        false,
        true
      )
    }
    return -1
  }
}

// Modify --------------------------------------------

// Replace -------------------------------------------

// Delete --------------------------------------------

// ---------------------------------------------------

/**
 * `MarkdownConfig` for syntax highlights related to the
 * Suggestion syntax. this includes:
 *  - insert block `++ ++`
 *  - insert inline `{++ ++}`
 *  - modify block `!! !> !!`
 *  - modify inline `{!! !> !!}`
 *  - replace block `~~ ~> ~~`
 *  - replace inline `{~~ ~> ~~}`
 *  - delete block `-- --`
 *  - delete inline `{-- --}`
 */
const StencilaSuggestionSyntax: MarkdownConfig = {
  defineNodes: [insertBlock, insertInline, insertBlockMark, insertInlineMark],
  parseBlock: [new InsertBlockParser()],
  parseInline: [new InsertInlineParser()],
  wrap: parseMixed((node) => {
    if (node.type.name === insertBlock.name) {
      const from = node.from + BLOCK_MARK_LENGTH
      const to = node.to - BLOCK_MARK_LENGTH
      if (from >= to) {
        return null
      }
      return {
        parser: markdownLanguage.parser,
        overlay: [{ from, to }],
      }
    }
    if (node.type.name === insertInline.name) {
      const from = node.from + INSERT_INLINE_OPEN.length
      const to = node.to - INSERT_INLINE_CLOSE.length
      return {
        parser: markdownLanguage.parser,
        overlay: [{ from, to }],
      }
    }
  }),
}

const highlightStyles: TagStyle[] = [
  {
    tag: customTags.insertMark,
    color: 'red',
  },
]

export { StencilaSuggestionSyntax, highlightStyles }
