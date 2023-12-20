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
  LeafBlockParser,
  LeafBlock,
} from '@lezer/markdown'

const customTags = {
  suggestMark: Tag.define(),
  suggestBase: Tag.define(),
}

// TODO ->
// modify inline, modify block,
// replace block, replace inline,
// delete block, delete inline

// Insert --------------------------------------------

const insertBlockRe = /^\+\+$/

// insert `NodeSpecs`
const insertBlock = {
  name: 'InsertBlock',
  block: true,
  style: customTags.suggestBase,
}
const insertInline = { name: 'InsertInline', style: customTags.suggestBase }
const insertBlockMark = {
  name: 'InsertBlockMark',
  style: customTags.suggestMark,
}
const insertInlineMark = {
  name: 'InsertInlineMark',
  style: customTags.suggestMark,
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
    if (!insertBlockRe.test(line.text)) {
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
      if (insertBlockRe.test(line.text)) {
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

const modOuterRe = /^!!$/
const modInnerRe = /^!>$/

const modBlockOuter = { name: 'ModifyBlockDelimiter' }
const modBlockInner = { name: 'ModifyBlockInner' }
const modInlineOuter = { name: 'ModifyInlineDelimiter' }
const modInlineInner = { name: 'ModifyInlineInner' }
const modMark = { name: 'ModifyMark', style: customTags.suggestMark }
// const modInlineMark = { n}

const MOD_BLOCK_DELIM = '!!'
const MOD_INNER_DELIM = '!>'
// const MOD_ININE_OPEN = '{!!'
// const MOD_ININE_CLOSE = '!!}'

const parseModifyLeaf = (
  cx: BlockContext,
  leaf: LeafBlock,
  nodeType: string,
  mark: string
): boolean => {
  const start = leaf.start
  const end = start + mark.length
  if (start >= end) {
    return false
  }
  console.log('hi')
  cx.addElement(
    cx.elt(nodeType, start, end, [cx.elt(modMark.name, start, end)])
  )
  console.log('hello')
  return true
}

/**
 * `LeafBlockParser` for the opening and closing delimiter
 * of a modify block.
 * eg: `!!`
 */
class ModifyBlockParser implements LeafBlockParser {
  nextLine = () => false
  finish(cx: BlockContext, leaf: LeafBlock): boolean {
    try {
      return parseModifyLeaf(cx, leaf, modBlockOuter.name, MOD_BLOCK_DELIM)
    } catch {
      return false
    }
  }
}

class ModifyInnerParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    try {
      return parseModifyLeaf(cx, leaf, modBlockInner.name, MOD_INNER_DELIM)
    } catch {
      return false
    }
  }
}

// class ModifyBlockCloseParser implements LeafBlockParser {}

// class ModifyInlineParser implements InlineParser {}

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
  defineNodes: [
    insertBlock,
    insertInline,
    insertBlockMark,
    insertInlineMark,
    modBlockInner,
    modBlockOuter,
    modInlineOuter,
    modInlineInner,
    modMark,
  ],
  parseBlock: [
    new InsertBlockParser(),
    // modify block parsers
    {
      name: modBlockOuter.name,
      leaf: (_, leaf) =>
        modOuterRe.test(leaf.content) ? new ModifyBlockParser() : null,
      endLeaf: (_, line) => !modOuterRe.test(line.text),
    },
    {
      name: modBlockInner.name,
      leaf: (_, leaf) =>
        modInnerRe.test(leaf.content) ? new ModifyInnerParser() : null,
      endLeaf: (_, line) => !modInnerRe.test(line.text),
    },
  ],
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
    tag: customTags.suggestMark,
    color: 'red',
  },
]

export { StencilaSuggestionSyntax, highlightStyles }
