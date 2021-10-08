import {
  inputRules,
  wrappingInputRule,
  textblockTypeInputRule,
  openDoubleQuote,
  closeDoubleQuote,
  openSingleQuote,
  closeSingleQuote,
  emDash,
  ellipsis,
  InputRule,
} from 'prosemirror-inputrules'
import { MarkType } from 'prosemirror-model'
import { articleSchema } from './schema'

/**
 * The ProseMirror `InputRules` (i.e. input macros) for a Stencila `Article`
 *
 * For docs and examples see:
 *  - https://prosemirror.net/docs/ref/#inputrules
 *  - https://github.com/ProseMirror/prosemirror-example-setup/blob/master/src/inputrules.js
 */
export const articleInputRules = inputRules({
  rules: [
    // Converts double dashes to an emdash
    emDash,

    // Converts three dots to an ellipsis character
    ellipsis,

    // “Smart” opening double quotes
    openDoubleQuote,

    // “Smart” closing double quotes
    closeDoubleQuote,

    // “Smart” opening single quotes
    openSingleQuote,

    // “Smart” closing single quotes
    closeSingleQuote,

    // Markdown emphasis
    markInputRule(/_(\S(?:|.*?\S))_$/, articleSchema.marks.Emphasis),

    // Markdown strong emphasis
    markInputRule(/\*{2}(\S(?:|.*?\S))\*{2}$/, articleSchema.marks.Strong),

    // Pandoc Markdown delete (strikeout)
    // TODO: Requires disambiguation with subscript rule (or removal)
    markInputRule(/~{2}(\S(?:|.*?\S))~{2}$/, articleSchema.marks.Delete),

    // Custom Markdown underline
    // TODO: Requires disambiguation with emphasis rule (or removal)
    markInputRule(/_{2}(\S(?:|.*?\S))_{2}$/, articleSchema.marks.Underline),

    // Pandoc Markdown subscript
    markInputRule(/~(\S(?:|.*?\S))~$/, articleSchema.marks.Subscript),

    // Pandoc Markdown superscript
    markInputRule(/\^(\S(?:|.*?\S))\^$/, articleSchema.marks.Superscript),

    // Markdown heading
    textblockTypeInputRule(
      /^(#{1,6})\s$/,
      articleSchema.nodes.Heading,
      (match) => ({ depth: match?.[1]?.length })
    ),

    // Markdown unordered list
    wrappingInputRule(/^\s*([-+*])\s$/, articleSchema.nodes.List, {
      order: 'Unordered',
    }),

    // Markdown ordered list
    wrappingInputRule(
      /^(\d+)\.\s$/,
      articleSchema.nodes.List,
      (match) => ({ order: match?.[1] }),
      (match, node) => node.childCount + node.attrs.order === match[1]
    ),

    // Markdown quote block
    wrappingInputRule(/^\s*>\s$/, articleSchema.nodes.QuoteBlock),
  ],
})

/**
 * Create a ProseMirror `Mark` around the matched text.
 *
 * Credits: https://discuss.prosemirror.net/t/input-rules-for-wrapping-marks/537/10
 */
function markInputRule(
  regexp: RegExp,
  markType: MarkType,
  getAttrs: Function | {} = {}
) {
  return new InputRule(regexp, (state, match, start, end) => {
    const attrs = getAttrs instanceof Function ? getAttrs(match) : getAttrs
    const tr = state.tr
    if (match[0] && match[1]) {
      const textStart = start + match[0].indexOf(match[1])
      const textEnd = textStart + match[1].length
      if (textEnd < end) tr.delete(textEnd, end)
      if (textStart > start) tr.delete(start, textStart)
      end = start + match[1].length
    }
    tr.addMark(start, end, markType.create(attrs))
    tr.removeStoredMark(markType) // Do not continue with mark.
    return tr
  })
}
