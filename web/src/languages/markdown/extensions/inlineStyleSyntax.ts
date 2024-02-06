import { InlineParser, MarkdownConfig } from '@lezer/markdown'

// const styleInlineRe = /\[[\S\s]*?\]{[\S\s]*?}({(\bcss\b)})?/

const styleInline = { name: 'StyleInline' }
// const styleInlinePar

class StyleInlineParse implements InlineParser {
  name = styleInline.name
  parse = (): number => {
    // const styleIndex = cx.slice(pos, cx.text.length - 1).search(styleInlineRe)

    return -1
  }
}

const StencilaStyleInlineSyntax: MarkdownConfig = {
  defineNodes: [styleInline],
  parseInline: [new StyleInlineParse()],
}

export { StencilaStyleInlineSyntax }
