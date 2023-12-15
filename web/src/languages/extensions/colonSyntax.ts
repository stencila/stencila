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

const openDelimiterIf = /^:::(:{1,10})?\s(\bif\b)/

const delimiterElse = /^:::(:{1,10})?\s(\belse\b)/

const delimiterElseIf = /^:::(:{1,10})?\s(\belif\b)/

const delimiterFor = /^:::(:{1,10})?\s(\bfor\b)/

// const delimiterStyle = /^:::(:{1,10})?\s((\bcss\b)\s)?{*}/

const closeDelimiter = /^:::(:{1,10})?$/

const customTags = {
  base: Tag.define(),
  colonDelim: Tag.define(),
  keyword: Tag.define(),
  codeStatement: Tag.define(),
}

const openIfStatement = { name: 'OpenIfStatement', style: customTags.base }
const elseIfStatement = { name: 'ElseIfStatement', style: customTags.base }
const openForStatement = { name: 'OpenForStatement', style: customTags.base }
const elseStatement = { name: 'ElseStatement', style: customTags.base }
const closingDelimiter = {
  name: 'ClosingColonDelimiter',
  style: customTags.base,
}

const delimiterMark = { name: 'DelimiterMark', style: customTags.colonDelim }
const keywordMark = { name: 'KeywordMark', style: customTags.keyword }
const condition = { name: 'Condition', style: customTags.codeStatement }

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
 * `LeafBlockParser` for parsing the opening of an `if` statement
 * eg `::: if true`
 */
class OpeningIfParser implements LeafBlockParser {
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

    // add condition el, runs from 1 after kw to the end of the leaf (line)
    const condStart = kwStart + kwLength + 1
    elements.push(
      cx.elt(condition.name, condStart, leaf.start + leaf.content.length)
    )

    cx.addLeafElement(
      leaf,
      cx.elt(openIfStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )

    return true
  }
}

/**
 * `LeafBLockParser` for parsing an 'Else if' statement
 *  eg: `:::: elif`
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
    // add condition el, runs from 1 after the kw element to the end of the leaf
    const condStart = kwEl.to + 1
    elements.push(
      cx.elt(condition.name, condStart, leaf.start + leaf.content.length)
    )

    cx.addLeafElement(
      leaf,
      cx.elt(openIfStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )
    return true
  }
}

/**
 *
 */
class OpeningForParser implements LeafBlockParser {
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

    // add condition el, runs from 1 after the kw element to the end of the leaf
    const statementStart = kwEl.to + 1
    const inIdx = leaf.content.indexOf(` ${IN} `) + 1

    if (inIdx === -1) {
      elements.push(
        cx.elt(condition.name, statementStart, leaf.start + leaf.content.length)
      )
    } else {
      const inStart = leaf.start + inIdx
      const inKwEl = cx.elt(keywordMark.name, inStart, inStart + IN.length)
      elements.push(
        cx.elt(
          condition.name,
          statementStart,
          leaf.start + leaf.content.length,
          [inKwEl]
        )
      )
    }

    cx.addLeafElement(
      leaf,
      cx.elt(openIfStatement.name, leaf.start, getLeafEnd(leaf), elements)
    )
    return true
  }
}

/**
 *  `LeafBlockParser` for parsing and else statement for an `if` or `for`
 *  eg: `::: else`
 */
class ElseParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
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
  }
}

/**
 *  `LeafBlockParser` for parsing the closing delimiter
 *  for `if`, `for` and `style` blocks
 *  eg `:::` | `:::::`
 */
class ClosingDelimiterParser implements LeafBlockParser {
  nextLine = () => false
  finish = (cx: BlockContext, leaf: LeafBlock): boolean => {
    cx.addLeafElement(
      leaf,
      cx.elt(
        closingDelimiter.name,
        leaf.start,
        leaf.start + leaf.content.length,
        [createDelimiter(cx, leaf.start, leaf.content.trim().length)]
      )
    )
    return true
  }
}

const StencilaColonSyntax: MarkdownConfig = {
  defineNodes: [
    openIfStatement,
    elseStatement,
    closingDelimiter,
    delimiterMark,
    keywordMark,
    condition,
  ],
  parseBlock: [
    {
      name: openIfStatement.name,
      leaf: (_, leaf) =>
        openDelimiterIf.test(leaf.content) ? new OpeningIfParser() : null,
      endLeaf: (_, line) => !openDelimiterIf.test(line.text),
    },
    {
      name: elseIfStatement.name,
      leaf: (_, leaf) =>
        delimiterElseIf.test(leaf.content) ? new ElseIfParser() : null,
      endLeaf: (_, line) => !delimiterElseIf.test(line.text),
    },
    {
      name: openForStatement.name,
      leaf: (_, leaf) =>
        delimiterFor.test(leaf.content) ? new OpeningForParser() : null,
      endLeaf: (_, line) => !delimiterFor.test(line.text),
    },
    {
      name: elseStatement.name,
      leaf: (_, leaf) =>
        delimiterElse.test(leaf.content) ? new ElseParser() : null,
      endLeaf: (_, line) => !delimiterElse.test(line.text),
    },
    {
      name: closingDelimiter.name,
      leaf: (_, leaf) =>
        closeDelimiter.test(leaf.content.trim())
          ? new ClosingDelimiterParser()
          : null,
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
