/**
 * Shared Tiptap extension set for Stencila edit views and tests.
 */
import type { Extensions } from '@tiptap/core'
import Blockquote from '@tiptap/extension-blockquote'
import Bold from '@tiptap/extension-bold'
import BulletList from '@tiptap/extension-bullet-list'
import CodeBlock from '@tiptap/extension-code-block'
import Document from '@tiptap/extension-document'
import Heading from '@tiptap/extension-heading'
import HorizontalRule from '@tiptap/extension-horizontal-rule'
import Italic from '@tiptap/extension-italic'
import ListItem from '@tiptap/extension-list-item'
import OrderedList from '@tiptap/extension-ordered-list'
import Paragraph from '@tiptap/extension-paragraph'
import { Table } from '@tiptap/extension-table'
import TableCell from '@tiptap/extension-table-cell'
import TableHeader from '@tiptap/extension-table-header'
import TableRow from '@tiptap/extension-table-row'
import Text from '@tiptap/extension-text'

import { History } from './history'
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
    // Document root
    EditableDocument,

    // Blocks
    Paragraph,
    Heading.configure({ levels: [1, 2, 3, 4, 5, 6] }),
    Blockquote,
    CodeBlock,
    HorizontalRule,

    // List structure
    BulletList,
    OrderedList,
    ListItem,

    // Table structure
    Table,
    TableRow,
    TableHeader,
    TableCell,

    // Text and inline marks
    Text,
    Bold,
    Italic,
    CodeMark,
    StrikeoutMark,
    UnderlineMark,
    SubscriptMark,
    SuperscriptMark,
    LinkMark,

    // Opaque Stencila nodes
    StencilaBlock,
    StencilaInline,

    // Editing behavior
    History,
  ]
}
