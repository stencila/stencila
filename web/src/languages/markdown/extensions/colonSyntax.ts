/**
 * CodeMirror syntax extensions for Stencila block nodes that
 * use Pandoc-style "fenced div" syntax (paragraphs beginning
 * with at least three colons)
 */

import { TagStyle } from '@codemirror/language'
import { Tag } from '@lezer/highlight'
import {
  BlockContext,
  LeafBlockParser,
  MarkdownConfig,
  LeafBlock,
  Element,
} from '@lezer/markdown'

const IF = 'if'
const ELIF = 'elif'
const ELSE = 'else'
const FOR = 'for'
const IN = 'in'

const delimiterIf = /^:::(:{1,10})?\s(\bif\b)/

const delimiterElse = /^:::(:{1,10})?\s(\belse\b)/

const delimiterElseIf = /^:::(:{1,10})?\s(\belif\b)/

const delimiterFor = /^:::(:{1,10})?\s(\bfor\b)/

const delimiterStyled = /^:::(:{1,10})?\s(\bcss\b\s)?{[\S\s]*?}/

const closeDelimiter = /^:::(:{1,10})?$/

const customTags = {
  base: Tag.define(),
  colonDelim: Tag.define(),
  keyword: Tag.define(),
  codeStatement: Tag.define(),
}

// `NodeSpec` objects for elements
const ifStatement = { name: 'IfStatement', style: customTags.base }
const elseIfStatement = { name: 'ElseIfStatement', style: customTags.base }
const forStatement = { name: 'ForStatement', style: customTags.base }
const elseStatement = { name: 'ElseStatement', style: customTags.base }
const styledStatement = { name: 'StyledStatement', style: customTags.base }
const closeStatement = {
  name: 'CloseStatement',
  style: customTags.base,
}

const delimiterMark = { name: 'DelimiterMark', style: customTags.colonDelim }
const keywordMark = { name: 'KeywordMark', style: customTags.keyword }
const codeStatement = { name: 'CodeStatement', style: customTags.codeStatement }

const createDelimiter = (
  cx: BlockContext,
  start: number,
  length: number
): Element => cx.elt(delimiterMark.name, start, start + length)

const createKeyWordEl = (
  cx: BlockContext,
  start: number,
  length: number
): Element => cx.elt(keywordMark.name, start, start + length)

const getLeafEnd = (leaf: LeafBlock): number =>
  leaf.start + leaf.content.trim().length

/**
 * `LeafBlockParser` for parsing the opening of an `if` block e.g.
 * `::: if a > 10`
 */
class IfParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    const elements = []
    const delimLength = leaf.content.trim().search(/\s/)
    if (delimLength === -1) {
      return false
    }
    elements.push(createDelimiter(cx, leaf.start, delimLength))

    // add le
    const kwStart = leaf.start + leaf.content.indexOf(IF)
    const kwLength = IF.length

    if (kwLength === -1) {
      return false
    }

    elements.push(createKeyWordEl(cx, kwStart, kwLength))

    // add CodeStatement el, runs from 1 after kw to the end of the leaf (line)
    const condStart = kwStart + kwLength + 1
    elements.push(
      cx.elt(codeStatement.name, condStart, leaf.start + leaf.content.length)
    )

    cx.addLeafElement(
      leaf,
      cx.elt(ifStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )

    return true
  }
}

/**
 * `LeafBLockParser` for parsing an 'else if' statement e.g.
 *  `::: elif x < 10`
 */
class ElseIfParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    const elements = []
    const delimLength = leaf.content.trim().search(/\s/)
    if (delimLength === -1) {
      return false
    }
    elements.push(createDelimiter(cx, leaf.start, delimLength))

    // add keyword el
    const kwStart = leaf.start + leaf.content.indexOf(ELIF)
    const kwLength = ELIF.length

    if (kwLength === -1) {
      return false
    }
    const kwEl = createKeyWordEl(cx, kwStart, kwLength)
    elements.push(kwEl)
    // add CodeStatement el, runs from 1 after the kw element to the end of the leaf
    const condStart = kwEl.to + 1
    elements.push(
      cx.elt(codeStatement.name, condStart, leaf.start + leaf.content.length)
    )

    cx.addLeafElement(
      leaf,
      cx.elt(elseIfStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )
    return true
  }
}

/**
 *  `LeafBLockParser` for parsing the opening of a 'for' block e.g.
 *  `::: for x in y`
 */
class ForParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    const elements = []
    const delimLength = leaf.content.trim().search(/\s/)
    if (delimLength === -1) {
      return false
    }
    elements.push(createDelimiter(cx, leaf.start, delimLength))

    // add keyword el
    const kwIndex = leaf.content.indexOf(FOR)
    const kwStart = leaf.start + kwIndex
    const kwLength = FOR.length

    if (kwStart === -1) {
      return false
    }

    const kwEl = createKeyWordEl(cx, kwStart, kwLength)
    elements.push(kwEl)

    // add codeStatement el, runs from 1 after the kw element to the end of the leaf
    const statementStart = kwEl.to + 1
    const inIdx = leaf.content.indexOf(` ${IN} `) + 1

    if (inIdx === -1) {
      elements.push(
        cx.elt(
          codeStatement.name,
          statementStart,
          leaf.start + leaf.content.length
        )
      )
    } else {
      const inStart = leaf.start + inIdx
      const inKwEl = cx.elt(keywordMark.name, inStart, inStart + IN.length)
      elements.push(
        cx.elt(
          codeStatement.name,
          statementStart,
          leaf.start + leaf.content.length,
          [inKwEl]
        )
      )
    }

    cx.addLeafElement(
      leaf,
      cx.elt(forStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )
    return true
  }
}

/**
 *  `LeafBlockParser` for parsing the `else` clause of an `if` or `for` block e.g.
 * `::: else`
 */
class ElseParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock) => {
    const delimLength = leaf.content.trim().search(/\s/)
    if (delimLength === -1) {
      return false
    }
    const kwStart = leaf.start + leaf.content.indexOf(ELSE)
    const kwLength = ELSE.length
    cx.addLeafElement(
      leaf,
      cx.elt(elseStatement.name, leaf.start, getLeafEnd(leaf), [
        createDelimiter(cx, leaf.start, delimLength),
        createKeyWordEl(cx, kwStart, kwLength),
      ])
    )
    return true
  }
}

/**
 *  `LeafBlockParser` for parsing the opening of a styled block e.g.
 * `::: {}` or `::: css {color: red;}`
 */
class StyledParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    const delimLength = leaf.content.trim().search(/\s/)
    if (delimLength === -1) {
      return false
    }

    const elements = [createDelimiter(cx, leaf.start, delimLength)]

    const hasLanguageSpec = /^:::(:{1,10})?\s(\bcss\b)\s/.test(leaf.content)

    if (hasLanguageSpec) {
      const langKwStart = leaf.start + delimLength + 1
      const langKwLength = leaf.content.substring(delimLength + 1).search(/\s/)
      if (langKwLength !== -1) {
        elements.push(createKeyWordEl(cx, langKwStart, langKwLength))
      }
    }

    const rulesStart = leaf.content.search('{')
    if (rulesStart !== -1) {
      elements.push(
        cx.elt(codeStatement.name, leaf.start + rulesStart, getLeafEnd(leaf))
      )
    }

    cx.addLeafElement(
      leaf,
      cx.elt(styledStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )

    return true
  }
}

/**
 *  `LeafBlockParser` for parsing the closing delimiter for `if`, `for` and
 *  styled blocks e.g.
 *  `:::` or `:::::`
 */
class CloseParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    cx.addLeafElement(
      leaf,
      cx.elt(closeStatement.name, leaf.start, getLeafEnd(leaf), [
        createDelimiter(cx, leaf.start, leaf.content.trim().length),
      ])
    )
    return true
  }
}

/**
 * `MarkdownConfig` to apply the necessary parsers for highlighting the
 * colon delimiter syntax as an `Extension` for the markdown language
 */
const StencilaColonSyntax: MarkdownConfig = {
  defineNodes: [
    ifStatement,
    elseIfStatement,
    forStatement,
    elseStatement,
    styledStatement,
    closeStatement,
    delimiterMark,
    keywordMark,
    codeStatement,
  ],
  parseBlock: [
    {
      name: ifStatement.name,
      leaf: (_, leaf) =>
        delimiterIf.test(leaf.content) ? new IfParser() : null,
      endLeaf: (_, line) => !delimiterIf.test(line.text),
    },
    {
      name: elseIfStatement.name,
      leaf: (_, leaf) =>
        delimiterElseIf.test(leaf.content) ? new ElseIfParser() : null,
      endLeaf: (_, line) => !delimiterElseIf.test(line.text),
    },
    {
      name: forStatement.name,
      leaf: (_, leaf) =>
        delimiterFor.test(leaf.content) ? new ForParser() : null,
      endLeaf: (_, line) => !delimiterFor.test(line.text),
    },
    {
      name: elseStatement.name,
      leaf: (_, leaf) =>
        delimiterElse.test(leaf.content) ? new ElseParser() : null,
      endLeaf: (_, line) => !delimiterElse.test(line.text),
    },
    {
      name: styledStatement.name,
      leaf: (_, leaf) =>
        delimiterStyled.test(leaf.content) ? new StyledParser() : null,
      endLeaf: (_, line) => !delimiterStyled.test(line.text),
    },
    {
      name: closeStatement.name,
      leaf: (_, leaf) =>
        closeDelimiter.test(leaf.content.trim()) ? new CloseParser() : null,
      endLeaf: (_, line) => !closeDelimiter.test(line.text.trim()),
    },
  ],
}

const COLON_SYNTAX_BG = 'rgba(0,0,0,0.1)'

const highlightStyles: TagStyle[] = [
  {
    tag: customTags.colonDelim,
    color: 'blue',
    fontWeight: '700',
    backgroundColor: COLON_SYNTAX_BG,
  },
  {
    tag: customTags.keyword,
    color: 'green',
    fontStyle: 'italic',
    backgroundColor: COLON_SYNTAX_BG,
  },
  {
    tag: customTags.codeStatement,
    color: 'purple',
    backgroundColor: COLON_SYNTAX_BG,
  },
  {
    tag: customTags.base,
    backgroundColor: COLON_SYNTAX_BG,
  },
]

export { StencilaColonSyntax, highlightStyles }
