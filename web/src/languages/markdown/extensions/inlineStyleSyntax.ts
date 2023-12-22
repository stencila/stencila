import { InlineParser, InlineContext, MarkdownConfig } from '@lezer/markdown'

const inlineStyleRe = /\[[\S\s]*?\]{[\S\s]*?}({(\bcss\b)})?/

const styleInline = { name: 'StyleInline' }
// const styleInlinePar

class StyleInlineParse implements InlineParser {
  name = styleInline.name
  parse = (cx: InlineContext, next: number, pos: number): number => {
    const styleIndex = cx.slice(pos, cx.text.length - 1).search(inlineStyleRe)
    if (styleIndex !== -1) {
      console.log(styleIndex)
    }
    return -1
  }
}

const StencilaInlineStyle: MarkdownConfig = {
  defineNodes: [styleInline],
  parseInline: [new StyleInlineParse()],
}

export { StencilaInlineStyle }
