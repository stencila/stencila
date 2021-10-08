import {
  toggleMark,
  wrapIn,
  setBlockType,
  chainCommands,
  exitCode,
  joinUp,
  joinDown,
  lift,
  selectParentNode,
  Keymap,
} from 'prosemirror-commands'
import { redo, undo } from 'prosemirror-history'
import { undoInputRule } from 'prosemirror-inputrules'
import {
  wrapInList,
  splitListItem,
  liftListItem,
  sinkListItem,
} from 'prosemirror-schema-list'
import { articleSchema } from './schema'

/**
 * The ProseMirror `KeyMap` (i.e. key bindings) for a Stencila `Article`.
 *
 * Most of these bindings are based on those in https://github.com/ProseMirror/prosemirror-example-setup.
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
  'Mod-i': toggleMark(articleSchema.marks.Emphasis),
  'Mod-b': toggleMark(articleSchema.marks.Strong),

  // Changing the type of blocks
  'Shift-Ctrl-0': setBlockType(articleSchema.nodes.Paragraph),
  'Shift-Ctrl-1': setBlockType(articleSchema.nodes.Heading, {depth: 1}),
  'Shift-Ctrl-2': setBlockType(articleSchema.nodes.Heading, {depth: 2}),
  'Shift-Ctrl-3': setBlockType(articleSchema.nodes.Heading, {depth: 3}),
  'Shift-Ctrl-4': setBlockType(articleSchema.nodes.Heading, {depth: 4}),
  'Shift-Ctrl-5': setBlockType(articleSchema.nodes.Heading, {depth: 5}),
  'Shift-Ctrl-6': setBlockType(articleSchema.nodes.Heading, {depth: 6}),

  // Wrapping blocks in another type
  'Ctrl->': wrapIn(articleSchema.nodes.QuoteBlock),
  
  // List creation / manipulation
  'Shift-Ctrl-8': wrapInList(articleSchema.nodes.List, {order: 'Unordered'}),
  'Shift-Ctrl-9': wrapInList(articleSchema.nodes.List, {order: 'Ascending'}),
  'Enter': splitListItem(articleSchema.nodes.ListItem),
  'Mod-[': liftListItem(articleSchema.nodes.ListItem),
  'Mod-]': sinkListItem(articleSchema.nodes.ListItem)
}
