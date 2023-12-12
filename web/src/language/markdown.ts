import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import {
  defaultHighlightStyle,
  HighlightStyle,
} from "@codemirror/language";
import { Tag, styleTags, tags } from '@lezer/highlight'
import { Line, MarkdownConfig } from '@lezer/markdown'

const nodes = {
  colonDelimiterMark: "ColonDelimiterMark",
  blockIf: "BlockIf",
  blockIfMark: "BlockIfMark"
}


const customTags = {
  colonDelimiterMark: Tag.define(),
  blockIf: Tag.define(),
  blockIfMark: Tag.define(),
}

/**
 * Check the line starts with the delimiter for if|for|style blocks
 * @param line instance of `@lezer/markdown`'s `Line` class
 * @returns 
 */
const hasStartIfDelimiter = (line: Line): boolean => /^:::(:{1,2})?\s(\b(if|elif|else)\b)/.test(line.text)

/**
 * Check is line contains the end delimiter ie. no trailing space
 * @param line instance of `@lezer/markdown`'s `Line` class
 * @returns 
 */
const hasEndDelimiter = (line: Line): boolean => /^:::(:{1,2})?$/.test(line.text.trim())

// const isNestedEndDelimiter = (line: Line): boolean => /^:::::$/.test(line.text.trim())

// const findIfBlockEnd = (context: BlockContext, line: Line, nestedLevel: number = 0): number => {
//   let hasNextLine: boolean
//   let lineHasCloseDelim: boolean
//   do {
//     hasNextLine = context.nextLine()
//     // find a nested if blocks OR else / elif statements
//     if (isDelimeterNestedIfBlock(line)) {
//       context.addElement(context.elt(nodes.colonDelimiterMark, context.lineStart, context.lineStart + 5))
//       findIfBlockEnd(context, line, nestedLevel + 1)
//       context.addElement(context.elt(nodes.colonDelimiterMark, context.lineStart, context.lineStart + 5))
//       context.nextLine()
//     } else if (isStartElseBLock(line)) {
//       const delimLength = nestedLevel > 0 ? 5 : 4
//       context.addElement(context.elt(nodes.colonDelimiterMark, context.lineStart, context.lineStart + delimLength))
//     }

//     lineHasCloseDelim = nestedLevel > 0 ? isNestedEndDelimiter(line) : isEndDelimiter(line) 
//   } while (hasNextLine && !lineHasCloseDelim)

//   if (!hasNextLine) {
//     return -1
//   }

//   return context.lineStart + 3
// }

const stencilaBlockConfig: MarkdownConfig = {
  defineNodes: [nodes.colonDelimiterMark, nodes.blockIf, nodes.blockIfMark],
  parseBlock: [{
    name: nodes.blockIf,
    parse: (context, line) => { 
      if (!hasStartIfDelimiter(line) && !hasEndDelimiter(line)) {
        return false
      }

      const from = context.lineStart

      // find index of first whitespace
      const wsIndex = line.text.trim().search(/\s/)
      
      const to = wsIndex === -1 ? from + line.text.length : from + wsIndex

      if (from >= to) {
        return false
      }

      const delimiter = context.elt(nodes.colonDelimiterMark, from, to)
      context.addElement(delimiter)

      // if th
      if (hasStartIfDelimiter(line)) {
        // find one of the keywords after 
        const ifMatches = line.text.substring(wsIndex + 1).match(/^\b(if|elif|else)\b/)
        if (ifMatches) {
          const keyWord = ifMatches[0]
          const kwFrom = delimiter.to
          const kwTo = kwFrom + keyWord.length + 1
          context.addElement(context.elt(nodes.blockIfMark, kwFrom, kwTo))
        }
      }
      // Send context to the next line before ending block
      context.nextLine()

      // add a if block element, which will create a wrap around the non delimiter or keyword text on the line
      context.addElement(context.elt(nodes.blockIf, from, context.prevLineEnd()))

      return true

      // // get the end of the block
      // const to = findIfBlockEnd(context, line)

      // if (to === -1) {
      //   return false
      // }

      // const ifElt = context.elt(nodes.blockIf, from, to)
      // context.addElement(ifElt)

      // context.addElement(context.elt(nodes.colonDelimiterMark, context.lineStart, to))
      // context.nextLine()
      // return true
    },
  }],
  props: [
    styleTags({
      [nodes.colonDelimiterMark]: customTags.colonDelimiterMark,
      [nodes.blockIfMark]: customTags.blockIfMark,
      [nodes.blockIf]: customTags.blockIf
    })
  ]
}

const ifBlockBG = 'rgba(0,0,0,0.1)'

const markDownHighlightStyle = HighlightStyle.define([
  ...defaultHighlightStyle.specs,
  {
    tag: tags.heading,
    fontWeight: 700,
    textDecoration: 'none'
  },
  {
    tag: customTags.colonDelimiterMark,
    color: 'blue',
    backgroundColor: ifBlockBG
  },
  {
    tag: customTags.blockIfMark,
    color: 'green',
    fontWeight: 600,
    fontStyle: 'italic',
    backgroundColor: ifBlockBG
  },
  {
    tag: customTags.blockIf,
    color: 'purple',
    backgroundColor: ifBlockBG
  },
]);


const stencilaMarkdown = () => markdown({ base: markdownLanguage, extensions: [stencilaBlockConfig]  })

export { stencilaMarkdown, markDownHighlightStyle }