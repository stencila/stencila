/**
 * Shared Tiptap extension set for Stencila edit views and tests.
 */
import type { Extensions } from '@tiptap/core'
import Bold from '@tiptap/extension-bold'
import Document from '@tiptap/extension-document'
import Heading from '@tiptap/extension-heading'
import Italic from '@tiptap/extension-italic'
import Paragraph from '@tiptap/extension-paragraph'
import Text from '@tiptap/extension-text'

import {
  CodeMark,
  LinkMark,
  StrikeoutMark,
  SubscriptMark,
  SuperscriptMark,
  UnderlineMark,
} from './marks'
import { StencilaBlock, StencilaInline } from './stencila'

const EditableDocument = Document.extend({
  content: 'block*',
})

/**
 * Create the Tiptap extensions used to edit Stencila document content.
 */
export function createStencilaTiptapExtensions(): Extensions {
  return [
    EditableDocument,
    Paragraph,
    Heading.configure({ levels: [1, 2, 3, 4, 5, 6] }),
    Text,
    Bold,
    Italic,
    CodeMark,
    StrikeoutMark,
    UnderlineMark,
    SubscriptMark,
    SuperscriptMark,
    LinkMark,
    StencilaBlock,
    StencilaInline,
  ]
}
