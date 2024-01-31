import { StreamLanguage, TagStyle } from '@codemirror/language'
import * as tex from '@codemirror/legacy-modes/mode/stex'
import { parseMixed } from '@lezer/common'
import { Tag } from '@lezer/highlight'
import {
  BlockContext,
  BlockParser,
  InlineContext,
  InlineParser,
  Line,
  MarkdownConfig,
} from '@lezer/markdown'

const texParser = StreamLanguage.define(tex.stexMath).parser

const customTags = {
  texMark: Tag.define(),
}

const texBlock = {
  name: 'TexBlock',
}

const texInline = {
  name: 'TexInline',
}

const texBlockMark = {
  name: 'TexBlockMark',
  style: customTags.texMark,
}

const texInlineMark = {
  name: 'TexInlineMark',
  style: customTags.texMark,
}

const TEXBLOCK_MARK = '$$'
const TEXINLINE_MARK = '$'

/**
 * `BlockParser` for a Tex math block, creates a full block element
 * where the content is parsed with the latex parser
 * eg:
 * ```
 * $$
 * /math stuff here/
 * $$
 * ```
 */
class TexBlockParser implements BlockParser {
  name = texBlock.name
  parse = (cx: BlockContext, line: Line) => {
    if (line.text.startsWith(TEXBLOCK_MARK)) {
      const start = cx.lineStart + line.pos
      cx.addElement(
        cx.elt(texBlockMark.name, start, start + TEXBLOCK_MARK.length)
      )

      let hasDelim = false
      while (cx.nextLine()) {
        if (line.text.startsWith(TEXBLOCK_MARK)) {
          hasDelim = true
          break
        }
      }
      if (!hasDelim) {
        return false
      }
      const end = cx.lineStart + line.pos + TEXBLOCK_MARK.length
      cx.addElement(cx.elt(texBlock.name, start, end))
      cx.addElement(cx.elt(texBlockMark.name, end - TEXBLOCK_MARK.length, end))
      cx.nextLine()
      return true
    }
    return false
  }
  endLeaf(_: unknown, line: Line): boolean {
    return line.text.startsWith(TEXBLOCK_MARK)
  }
}

// inline delims object
const texInlineDelims = { resolve: texInline.name, mark: texInlineMark.name }

/**
 * `InlineParser`
 */
class TexInlineParser implements InlineParser {
  name = texInline.name
  parse = (cx: InlineContext, next: number, pos: number): number => {
    const delimCharCode = TEXINLINE_MARK.charCodeAt(0)
    if (next !== delimCharCode || cx.char(pos + 1) === delimCharCode) {
      return -1
    }
    return cx.addDelimiter(texInlineDelims, pos, pos + 1, true, true)
  }
}

/**
 * creates nested parsers for the content of the math blocks
 */
const mathParser = parseMixed((node) => {
  // parse inline tex math
  if (node.type.name === texInline.name) {
    const markLength = TEXINLINE_MARK.length
    return {
      parser: texParser,
      overlay: [{ from: node.from + markLength, to: node.to - markLength }],
    }
  }
  // parse block tex math
  if (node.type.name === texBlock.name) {
    const markLength = TEXBLOCK_MARK.length
    const start = node.from + markLength
    const end = node.to - markLength
    console.log(node.type)

    if (start >= end) {
      return null
    }
    return {
      parser: texParser,
      overlay: [{ from: start, to: end }],
    }
  }
  return null
})

/**
 * `Extension` for of various math syntax blocks
 */
const StencilaMathSyntax: MarkdownConfig = {
  defineNodes: [texBlock, texInline, texBlockMark, texInlineMark],
  parseBlock: [new TexBlockParser()],
  parseInline: [new TexInlineParser()],
  wrap: mathParser,
}

const highlightStyles: TagStyle[] = [
  {
    tag: customTags.texMark,
    color: '#00FF00',
  },
]

export { StencilaMathSyntax, highlightStyles }
