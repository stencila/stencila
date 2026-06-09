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
import { TaskItem, TaskList } from '@tiptap/extension-list'
import ListItem from '@tiptap/extension-list-item'
import OrderedList from '@tiptap/extension-ordered-list'
import Paragraph from '@tiptap/extension-paragraph'
import { Table } from '@tiptap/extension-table'
import TableCell from '@tiptap/extension-table-cell'
import TableHeader from '@tiptap/extension-table-header'
import TableRow from '@tiptap/extension-table-row'
import Text from '@tiptap/extension-text'

import { History } from './history'
import { MathBlock, MathInline } from './math'
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

const EditableCodeBlock = CodeBlock.extend({
  addAttributes() {
    return {
      ...(this.parent?.() ?? {}),
      id: {
        default: null,
        rendered: false,
      },
      isDemo: {
        default: null,
        rendered: false,
      },
    }
  },
})

const EditableTable = Table.extend({
  addAttributes() {
    return {
      ...(this.parent?.() ?? {}),
      id: {
        default: null,
        rendered: false,
      },
      label: {
        default: null,
        rendered: false,
      },
      labelAutomatically: {
        default: null,
        rendered: false,
      },
      caption: {
        default: null,
        rendered: false,
      },
      notes: {
        default: null,
        rendered: false,
      },
    }
  },
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
    EditableCodeBlock,
    MathBlock,
    HorizontalRule,

    // List structure
    BulletList,
    OrderedList,
    ListItem,
    TaskList,
    TaskItem.configure({ nested: true }),

    // Table structure
    EditableTable,
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
    MathInline,

    // Opaque Stencila nodes
    StencilaBlock,
    StencilaInline,

    // Editing behavior
    History,
  ]
}
