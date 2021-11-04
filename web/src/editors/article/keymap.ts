import {
  joinDown,
  joinUp,
  Keymap,
  lift,
  selectParentNode,
  setBlockType,
  toggleMark,
  wrapIn,
} from 'prosemirror-commands'
import { redo, undo } from 'prosemirror-history'
import { undoInputRule } from 'prosemirror-inputrules'
import {
  liftListItem,
  sinkListItem,
  splitListItem,
  wrapInList,
} from 'prosemirror-schema-list'
import { articleSchema } from './schema'

/**
 * The ProseMirror `KeyMap` (i.e. key bindings) for a Stencila `Article`.
 *
 * Most of these bindings are based on those in https://github.com/ProseMirror/prosemirror-example-setup.
 * `Mod-` is a shorthand for `Cmd-` on Mac and `Ctrl-` on other platforms.
 *
 * For docs and examples see:
 *  - https://prosemirror.net/docs/ref/#keymap
 *  - https://github.com/ProseMirror/prosemirror-example-setup/blob/master/src/keymap.js
 */
// prettier-ignore
export const articleKeymap: Keymap = {
  // History
  'Mod-z': undo,
  'Backspace': undoInputRule,
  'Shift-Mod-z': redo,

  // Node navigation
  'Alt-ArrowUp': joinUp,
  'Alt-ArrowDown': joinDown,
  'Mod-BracketLeft': lift,
  'Escape': selectParentNode,

  // Toggling marks
  // These are consistent with Google Docs (and others?)
  'Mod-i': toggleMark(articleSchema.marks.Emphasis),
  'Mod-b': toggleMark(articleSchema.marks.Strong),
  'Mod-u': toggleMark(articleSchema.marks.NontextualAnnotation),
  'Alt-Shift-5': toggleMark(articleSchema.marks.Delete),
  'Mod-.': toggleMark(articleSchema.marks.Superscript),
  'Mod-,': toggleMark(articleSchema.marks.Subscript),

  // Changing the type of blocks
  'Shift-Mod-0': setBlockType(articleSchema.nodes.Paragraph),
  'Shift-Mod-1': setBlockType(articleSchema.nodes.Heading, {depth: 1}),
  'Shift-Mod-2': setBlockType(articleSchema.nodes.Heading, {depth: 2}),
  'Shift-Mod-3': setBlockType(articleSchema.nodes.Heading, {depth: 3}),
  'Shift-Mod-4': setBlockType(articleSchema.nodes.Heading, {depth: 4}),
  'Shift-Mod-5': setBlockType(articleSchema.nodes.Heading, {depth: 5}),
  'Shift-Mod-6': setBlockType(articleSchema.nodes.Heading, {depth: 6}),
  'Shift-Mod-\\': setBlockType(articleSchema.nodes.CodeBlock),
  
  // Wrapping blocks in another type
  'Mod->': wrapIn(articleSchema.nodes.QuoteBlock),
  
  // List creation / manipulation
  'Shift-Mod-8': wrapInList(articleSchema.nodes.List, {order: 'Unordered'}),
  'Shift-Mod-9': wrapInList(articleSchema.nodes.List, {order: 'Ascending'}),
  'Enter': splitListItem(articleSchema.nodes.ListItem),
  'Mod-[': liftListItem(articleSchema.nodes.ListItem),
  'Mod-]': sinkListItem(articleSchema.nodes.ListItem)
}
