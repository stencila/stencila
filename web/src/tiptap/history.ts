/**
 * Undo/redo support for the Stencila Tiptap editor.
 *
 * Tiptap's ProseMirror package already includes the history primitives, so this
 * local extension wraps them as ordinary Tiptap commands and keybindings.
 */
import { Extension, type RawCommands } from '@tiptap/core'
import {
  history,
  redo as redoHistory,
  undo as undoHistory,
} from '@tiptap/pm/history'
import type { Plugin } from '@tiptap/pm/state'

/**
 * Identify the ProseMirror history plugin within an editor's plugin list.
 *
 * The plugin created by `@tiptap/pm/history` carries its `depth` and
 * `newGroupDelay` configuration on `spec.config`. Matching on that shape lets
 * the sync client and tests locate the history plugin (to inspect or reset undo
 * state) without depending on plugin ordering.
 */
export function isHistoryPlugin(plugin: Plugin): boolean {
  const config = plugin.spec.config

  return (
    typeof config === 'object' &&
    config !== null &&
    'depth' in config &&
    'newGroupDelay' in config
  )
}

export interface HistoryOptions {
  /**
   * The maximum number of history events retained by ProseMirror.
   */
  depth: number

  /**
   * The delay, in milliseconds, before adjacent edits start a new undo group.
   */
  newGroupDelay: number
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    history: {
      /**
       * Undo the latest editable document change.
       */
      undo: () => ReturnType

      /**
       * Redo the latest undone document change.
       */
      redo: () => ReturnType
    }
  }
}

/**
 * History extension with standard platform editor shortcuts.
 */
export const History = Extension.create<HistoryOptions>({
  name: 'history',

  addOptions() {
    return {
      depth: 100,
      newGroupDelay: 500,
    }
  },

  addCommands() {
    return {
      undo:
        () =>
        ({ dispatch, editor, tr, view }) => {
          const didUndo = undoHistory(
            editor.state,
            dispatch ? (transaction) => view.dispatch(transaction) : undefined,
            view
          )

          if (didUndo && dispatch) {
            tr.setMeta('preventDispatch', true)
          }

          return didUndo
        },

      redo:
        () =>
        ({ dispatch, editor, tr, view }) => {
          const didRedo = redoHistory(
            editor.state,
            dispatch ? (transaction) => view.dispatch(transaction) : undefined,
            view
          )

          if (didRedo && dispatch) {
            tr.setMeta('preventDispatch', true)
          }

          return didRedo
        },
    } satisfies Partial<RawCommands>
  },

  addKeyboardShortcuts() {
    return {
      'Mod-z': () => this.editor.commands.undo(),
      'Mod-y': () => this.editor.commands.redo(),
      'Mod-Shift-z': () => this.editor.commands.redo(),
    }
  },

  addProseMirrorPlugins() {
    return [
      history({
        depth: this.options.depth,
        newGroupDelay: this.options.newGroupDelay,
      }),
    ]
  },
})
