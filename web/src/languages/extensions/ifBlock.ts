
import { TagStyle } from '@codemirror/language'
import { Tag } from '@lezer/highlight'
import { BlockContext, Element, BlockParser, NodeSpec  } from '@lezer/markdown'

const ifOpenDelimiter = /^:::(:{1,2})?\s(\b(if)\b)/

// const forOpenDelimiter = /^:::(:{1,2})?\s(\b(for)\b)/

const closeDelimiter = /^:::(:{1,2})?$/

const elseElifDelimiter = /^:::(:{1,2})?\s(\b(else|elif)\b)/


const hasOpenDelimiterIf = (str) => ifOpenDelimiter.test(str)

const hasElifElseDelimiter = (str) => elseElifDelimiter.test(str)

// const hasOpenDelimiterFor = (str) => forOpenDelimiter.test(str)

const hasEndDelimiter = (str) => closeDelimiter.test(str)

const customTags = {
  colonDelimiterMark: Tag.define(),
  blockIf: Tag.define(),
  blockIfStart: Tag.define(),
  logic: Tag.define(),
  keyWord: Tag.define()
}

const nodes: {[k: string]: NodeSpec } = {
  blockIf: { 
    name: "BlockIf", 
    block: true,
  },
  blockFor: { name: "BlockFor", block: true },
  delimiter: { name: "Delimiter", style: customTags.colonDelimiterMark },
  blockCondition: { name: "BlockCondition" },

  // new ones
  blockIfStart: { name: "BlockIfStart", block: true, style: customTags.blockIfStart },
  blockIfElif: { name: "BlockIfElif", block: true },
  blockIfElse: { name: "BlockIfElse", block: true },
  blockIfEnd: { name: "BlockIfEnd", block: true },
  keyWord: { name: "KeyWord", style: customTags.keyWord },
  logic: { name: "Logic", style: customTags.logic }
}


const newDelimiter = (cx: BlockContext, start: number, end: number): Element => 
  cx.elt("Delimiter", start, end)

const IfBlockParser: BlockParser = {
  name: "BlockIf",
  parse:  (cx, line) => {
    
    if(!hasOpenDelimiterIf(line.text)) {
      return false
    }
    const blockStart = cx.lineStart + line.pos

    // delimiter length to first 
    const delimeterLength = line.text.trim().search(/\s/)

    const elements = [newDelimiter(cx, blockStart, blockStart + delimeterLength)]
    let blockEnd: number
    while (cx.nextLine()) {
      if (hasElifElseDelimiter(line.text)) {
        const start = cx.lineStart + line.pos
        const wsIndex = line.text.trim().search(/\s/)
        elements.push(
          newDelimiter(cx, start, start + wsIndex)  
        )
      } else if (hasEndDelimiter(line.text) && line.text.length === delimeterLength) {
        const endLineStart = cx.lineStart + line.pos
        blockEnd = endLineStart + line.text.length
        elements.push(
          cx.elt("Delimiter", endLineStart, blockEnd)
        )
        cx.addElement(cx.elt("BlockIf", blockStart, blockEnd, elements))
        break
      }
    }
    cx.nextLine()
    return true
  }
}


const OpenIfBlockParser: BlockParser = {
  name: nodes.blockIfStart.name,
  parse: (cx, line) => {
    if(!hasOpenDelimiterIf(line.text)) {
      return false
    }
    const blockStart = cx.lineStart + line.pos
    const delimiterLength = line.text.trim().search(/\s/)
    if (delimiterLength === -1) {
      return false
    }
    const kwLength = 2
    const delimiter = cx.elt("Delimiter", blockStart, blockStart + delimiterLength)
    const pattern = /^\s(\bif\b)\s/
    if (pattern.test(line.text.substring(delimiterLength))) {
      const kwStart = blockStart + delimiterLength + 1
      const keyWord = cx.elt(nodes.keyWord.name, kwStart, kwStart + kwLength)
      const condition = cx.elt(nodes.logic.name, keyWord.to + 1, blockStart + line.text.length)
      cx.addElement(
        cx.elt(
          nodes.blockIfStart.name,
          blockStart,
          blockStart + line.text.length,
          [delimiter, keyWord, condition]
        )
      )
      cx.nextLine()
      return true
    }
    return false
  }
}

const EndIfBlockParser: BlockParser = {
  name: nodes.blockIfEnd.name,
  parse: (cx, line) => {
    if(!hasEndDelimiter(line.text.trim())) {
      return false
    }
    console.log(cx.parser)
    const blockStart = cx.lineStart + line.pos
    // const delimiterLength = line.text.trim().search(/\s/)
    cx.addElement(
      cx.elt(
        nodes.blockIfEnd.name,
        blockStart, 
        blockStart + line.text.length,
        [cx.elt(nodes.delimiter.name, blockStart, blockStart + line.text.length)]
      )
    )
    cx.nextLine()
    return true
  },
  after: nodes.blockIfStart.name,
}


const ifBlockBG = 'rgba(0,0,0,0.1)'


const ifBlockHighlightStyles: TagStyle[] = [
  {
    tag: customTags.colonDelimiterMark,
    color: 'blue',
    backgroundColor: ifBlockBG
  },
  {
    tag: customTags.blockIfStart,
    backgroundColor: ifBlockBG
  },
  {
    tag: customTags.keyWord,
    color: "green",
    fontStyle: 'italic',
    fontWeight: 600,
    backgroundColor: ifBlockBG
  },
  {
    tag: customTags.logic,
    color: 'purple',
    backgroundColor: ifBlockBG
  }
]

const ifBlockNodeList = Object.values(nodes)

const parsers = [OpenIfBlockParser, EndIfBlockParser]

export { IfBlockParser, ifBlockHighlightStyles, ifBlockNodeList, parsers }


