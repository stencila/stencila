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

import { hasOpeningDelimitir } from '../utilty'

const customTags = {
  suggestMark: Tag.define(),
  suggestBase: Tag.define(),
}

/**
 * Parse a simple block marker leaf element using provided
 * node and marker types
 * @param cx
 * @param leaf
 * @param nodeType
 * @param markType
 * @param mark
 * @returns
 */
const parseLeaf = (
  cx: BlockContext,
  leaf: LeafBlock,
  nodeType: string,
  markType: string,
  mark: string
): boolean => {
  const start = leaf.start
  const end = start + mark.length
  if (start >= end) {
    return false
  }
  cx.addLeafElement(
    leaf,
    cx.elt(nodeType, start, end, [cx.elt(markType, start, end)])
  )
  return true
}

/**
 * `LeafBlockParser` for the delimiters
 * of a suggest type block
 * eg: `!!`, `--`, `~~`
 */
class SuggestLeafParser implements LeafBlockParser {
  delimiter: string
  nodeType: string
  markType: string

  constructor(delim: string, nodeType, markType) {
    this.delimiter = delim
    ;(this.nodeType = nodeType), (this.markType = markType)
  }

  nextLine = () => false

  finish(cx: BlockContext, leaf: LeafBlock): boolean {
    try {
      return parseLeaf(cx, leaf, this.nodeType, this.markType, this.delimiter)
    } catch {
      return false
    }
  }
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
      cx.slice(pos, pos + INSERT_INLINE_CLOSE.length) === INSERT_INLINE_CLOSE &&
      cx.slice(pos - 3, pos) !== INSERT_INLINE_OPEN // <- check that there is content between the two delims
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

// modify `NodeSpecs`
const modBlockOuter = { name: 'ModifyBlockDelimiter' }
const modBlockInner = { name: 'ModifyBlockInner' }
const modInline = { name: 'ModifyInlineDelimiter' }
const modInlineInner = { name: 'ModifyInlineInner' }
const modMark = { name: 'ModifyMark', style: customTags.suggestMark }

const MOD_BLOCK_DELIM = '!!'
const MOD_INNER_DELIM = '!>'
const MOD_INLINE_OPEN = '{!!'
const MOD_INLINE_CLOSE = '!!}'

const modifyLeafParser = new SuggestLeafParser(
  MOD_BLOCK_DELIM,
  modBlockOuter.name,
  modMark.name
)

/**
 * `BlockParser` for the opening and closing delimiter
 * of a modify block.
 * eg: `!!`
 */
class ModifyBlockParser implements BlockParser {
  name = modBlockOuter.name
  leaf = (_, leaf) => (modOuterRe.test(leaf.content) ? modifyLeafParser : null)
  endLeaf = (_, line) => !modOuterRe.test(line.text)
}

const modifyInnerLeafParser = new SuggestLeafParser(
  MOD_INNER_DELIM,
  modBlockInner.name,
  modMark.name
)

/**
 * BlockParser` for the inner delimiter
 * of a modify block.
 * eg: `!>`
 */
class ModifyInnerParser implements BlockParser {
  name = modBlockInner.name
  leaf = (_, leaf) =>
    modInnerRe.test(leaf.content) ? modifyInnerLeafParser : null
  endLeaf = (_, line) => !modInnerRe.test(line.text)
}

/**
 * `InlineParser` for the modify inline delimiters,
 * creates a new element for each delimiter
 * eg: `{!! !> !!}`
 */
class ModifyInlineParser implements InlineParser {
  name = modInline.name
  parse = (cx: InlineContext, _next: number, pos: number): number => {
    if (cx.slice(pos, pos + MOD_INLINE_OPEN.length) === MOD_INLINE_OPEN) {
      // create open delim element
      return cx.addElement(
        cx.elt(modInline.name, pos, pos + MOD_INLINE_OPEN.length, [
          cx.elt(modMark.name, pos, pos + MOD_INLINE_OPEN.length),
        ])
      )
    } else if (
      cx.slice(pos, pos + MOD_INLINE_CLOSE.length) === MOD_INLINE_CLOSE &&
      hasOpeningDelimitir(cx, pos, MOD_INLINE_OPEN, MOD_INLINE_CLOSE)
    ) {
      // create closing delim element
      return cx.addElement(
        cx.elt(modInline.name, pos, pos + MOD_INLINE_CLOSE.length, [
          cx.elt(modMark.name, pos, pos + MOD_INLINE_CLOSE.length),
        ])
      )
    } else if (
      cx.slice(pos, pos + MOD_INNER_DELIM.length) === MOD_INNER_DELIM &&
      hasOpeningDelimitir(cx, pos, MOD_INLINE_OPEN, MOD_INNER_DELIM)
    ) {
      // create inner delim element
      return cx.addElement(
        cx.elt(modInlineInner.name, pos, pos + MOD_INNER_DELIM.length, [
          cx.elt(modMark.name, pos, pos + MOD_INNER_DELIM.length),
        ])
      )
    }
    return -1
  }
}

// Replace -------------------------------------------

const repOuterRe = /^~~$/
const repInnerRe = /^~>$/

// Replace `NodeSpecs`
const repBlockOuter = { name: 'ReplaceBlockDelimiter' }
const repBlockInner = { name: 'ReplaceBlockInner' }
const repInline = { name: 'ReplaceInlineDelimiter' }
const repInlineInner = { name: 'ReplaceInlineInner' }
const repMark = { name: 'ReplaceMark', style: customTags.suggestMark }

const REP_BLOCK_DELIM = '~~'
const REP_INNER_DELIM = '~>'
const REP_INLINE_OPEN = '{~~'
const REP_INLINE_CLOSE = '~~}'

const replaceBlockLeaf = new SuggestLeafParser(
  REP_BLOCK_DELIM,
  repBlockOuter.name,
  repMark.name
)

/**
 * `BlockParser` for the inner delimiter
 * of a replace block.
 * eg: `~>`
 */
class ReplaceBlockParser implements BlockParser {
  name = repBlockOuter.name
  leaf = (_, leaf) => (repOuterRe.test(leaf.content) ? replaceBlockLeaf : null)
  endLeaf = (_, line) => !repOuterRe.test(line.text)
}

const replaceInnerLeaf = new SuggestLeafParser(
  REP_INNER_DELIM,
  repBlockInner.name,
  repMark.name
)

/**
 * `BlockParser` for the inner delimiter
 * of a replace block.
 * eg: `~>`
 */
class ReplaceInnerParser implements BlockParser {
  name = repBlockInner.name
  leaf = (_, leaf) => (repInnerRe.test(leaf.content) ? replaceInnerLeaf : null)
  endLeaf = (_, line) => !repInnerRe.test(line.text)
}

/**
 * `InlineParser` for the inline replace markers
 * eg `{~~ ~> ~~}`
 */
class ReplaceInlineParser implements InlineParser {
  name = repInline.name
  parse = (cx: InlineContext, _next: number, pos: number): number => {
    if (cx.slice(pos, pos + REP_INLINE_OPEN.length) === REP_INLINE_OPEN) {
      return cx.addElement(
        cx.elt(repInline.name, pos, pos + REP_INLINE_OPEN.length, [
          cx.elt(repMark.name, pos, pos + REP_INLINE_OPEN.length),
        ])
      )
    } else if (
      cx.slice(pos, pos + REP_INLINE_CLOSE.length) === REP_INLINE_CLOSE &&
      hasOpeningDelimitir(cx, pos, REP_INLINE_OPEN, REP_INLINE_CLOSE)
    ) {
      return cx.addElement(
        cx.elt(repInline.name, pos, pos + REP_INLINE_CLOSE.length, [
          cx.elt(repMark.name, pos, pos + REP_INLINE_CLOSE.length),
        ])
      )
    } else if (
      cx.slice(pos, pos + REP_INNER_DELIM.length) === REP_INNER_DELIM &&
      hasOpeningDelimitir(cx, pos, REP_INLINE_OPEN, REP_INNER_DELIM)
    ) {
      return cx.addElement(
        cx.elt(repInlineInner.name, pos, pos + REP_INNER_DELIM.length, [
          cx.elt(repMark.name, pos, pos + REP_INNER_DELIM.length),
        ])
      )
    }
    return -1
  }
}

// Delete --------------------------------------------

const delDelimRe = /^--$/

const delBlock = { name: 'ReplaceBlock' }
const delInline = { name: 'DeleteInline' }
const delMark = { name: 'DeleteMark', style: customTags.suggestMark }

const DELETE_BLOCK = '--'
const DELETE_INLINE_OPEN = '{--'
const DELETE_INLINE_CLOSE = '--}'

const deleteBlockLeaf = new SuggestLeafParser(
  DELETE_BLOCK,
  delBlock.name,
  delMark.name
)

/**
 * `BlockParser` for the delete markers
 * eg `{-- --}`
 */
class DeleteBlockParser implements BlockParser {
  name = delBlock.name
  leaf = (_, leaf) => {
    return delDelimRe.test(leaf.content.trim()) ? deleteBlockLeaf : null
  }
  endLeaf = (_, line) => !delDelimRe.test(line.text)
}

/**
 * `InlineParser` for the inline replace markers
 * eg `{-- --}`
 */
class DeleteInlineParser implements InlineParser {
  name = delInline.name
  parse = (cx: InlineContext, next: number, pos: number): number => {
    if (cx.slice(pos, pos + DELETE_INLINE_OPEN.length) === DELETE_INLINE_OPEN) {
      return cx.addElement(
        cx.elt(delInline.name, pos, pos + DELETE_INLINE_OPEN.length, [
          cx.elt(delMark.name, pos, pos + DELETE_INLINE_OPEN.length),
        ])
      )
    } else if (
      cx.slice(pos, pos + DELETE_INLINE_CLOSE.length) === DELETE_INLINE_CLOSE &&
      hasOpeningDelimitir(cx, pos, DELETE_INLINE_OPEN, DELETE_INLINE_CLOSE)
    ) {
      return cx.addElement(
        cx.elt(delInline.name, pos, pos + DELETE_INLINE_CLOSE.length, [
          cx.elt(delMark.name, pos, pos + DELETE_INLINE_CLOSE.length),
        ])
      )
    }
    return -1
  }
}

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
    modInline,
    modInlineInner,
    modMark,
    repBlockInner,
    repBlockOuter,
    repInline,
    repInlineInner,
    repMark,
    delBlock,
    delInline,
    delMark,
  ],
  parseBlock: [
    new InsertBlockParser(),
    new ModifyBlockParser(),
    new ModifyInnerParser(),
    new ReplaceBlockParser(),
    new ReplaceInnerParser(),
    new DeleteBlockParser(),
  ],
  parseInline: [
    new InsertInlineParser(),
    new ModifyInlineParser(),
    new ReplaceInlineParser(),
    new DeleteInlineParser(),
  ],
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
    color: '#800000',
  },
]

export { StencilaSuggestionSyntax, highlightStyles }
