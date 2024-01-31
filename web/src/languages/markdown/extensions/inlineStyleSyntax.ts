import { InlineParser, InlineContext, MarkdownConfig } from '@lezer/markdown'

const styleInlineRe = /\[[\S\s]*?\]{[\S\s]*?}({(\bcss\b)})?/

const styleInline = { name: 'StyleInline' }
// const styleInlinePar

class StyleInlineParse implements InlineParser {
  name = styleInline.name
  parse = (cx: InlineContext, _next: number, pos: number): number => {
    const styleIndex = cx.slice(pos, cx.text.length - 1).search(styleInlineRe)
    if (styleIndex !== -1) {
      console.log(styleIndex)
    }
    return -1
  }
}

const StencilaStyleInlineSyntax: MarkdownConfig = {
  defineNodes: [styleInline],
  parseInline: [new StyleInlineParse()],
}

export { StencilaStyleInlineSyntax }
