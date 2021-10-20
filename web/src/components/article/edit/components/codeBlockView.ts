import { EditorView as CMEditorView } from '@codemirror/view'
import '@stencila/components'
import { Keymap } from '@stencila/components/dist/types/components/editor/editor'
import isEqual from 'lodash.isequal'
import { exitCode } from 'prosemirror-commands'
import { redo, undo } from 'prosemirror-history'
import { Node } from 'prosemirror-model'
import { Selection, TextSelection } from 'prosemirror-state'
import { EditorView, NodeView } from 'prosemirror-view'
import { articleSchema } from '../schema'

/**
 * Generate a `NodeView` to represent a Stencila `CodeBlock`
 *
 * Based on https://prosemirror.net/examples/codemirror/ and https://github.com/ProseMirror/prosemirror-schema-basic/blob/b5ae707ab1be98a1d8735dfdc7d1845bcd126f18/src/schema-basic.js#L59
 */
export class CodeBlockView implements NodeView {
  cm: CMEditorView | null = null
  dom: HTMLStencilaEditorElement
  getPos: () => number
  ignoreMutation?: NodeView['ignoreMutation']
  incomingChanges: boolean
  updating: boolean
  view: EditorView
  node: Node

  constructor(node: Node, view: EditorView, getPos: boolean | (() => number)) {
    this.node = node
    this.view = view
    if (typeof getPos === 'boolean') {
      this.getPos = () => 0
    } else {
      this.getPos = getPos
    }
    this.incomingChanges = false

    this.dom = document.createElement('stencila-editor')

    this.dom.contents = node.textContent
    this.dom.keymap = this.codeMirrorKeymap()
    this.dom.activeLanguage =
      typeof node.attrs.programmingLanguage === 'string'
        ? node.attrs.programmingLanguage
        : ''

    this.dom.addEventListener('setLanguage', (e) => {
      const evt = e as CustomEvent<{ name: string }>

      const languageChangeTransaction = this.view.state.tr.setNodeMarkup(
        this.getPos(),
        undefined,
        {
          programmingLanguage: evt.detail.name,
        }
      )

      this.view.dispatch(languageChangeTransaction)
    })

    this.dom
      .getRef()
      .then((ref) => {
        this.cm = ref
      })
      .catch((err) => {
        console.log(err)
      })

    // This flag is used to avoid an update loop between the outer and inner editor
    this.updating = false

    this.dom.contentChangeHandler = () => {
      this.valueChanged()
    }

    this.dom.addEventListener('focusin', () => {
      this.forwardSelection()
    })
  }

  codeMirrorKeymap = (): Keymap[] => {
    const view = this.view
    const dispatch = view.dispatch.bind(this)

    return [
      { key: 'ArrowUp', run: () => this.maybeEscape(-1) },
      { key: 'ArrowLeft', run: () => this.maybeEscape(-1) },
      { key: 'ArrowDown', run: () => this.maybeEscape(1) },
      { key: 'ArrowRight', run: () => this.maybeEscape(1) },
      {
        key: 'Ctrl-Enter',
        run: () => {
          if (exitCode(view.state, dispatch)) {
            view.focus()
          }
          return true
        },
      },
      {
        key: 'Mod-z',
        preventDefault: true,
        run: () => {
          undo(view.state, dispatch)
          return true
        },
      },
      {
        key: 'Shift-Mod-z',
        preventDefault: true,
        run: () => {
          redo(view.state, dispatch)
          return true
        },
      },
      {
        key: 'Mod-Y',
        preventDefault: true,
        run: () => {
          redo(view.state, dispatch)
          return true
        },
      },
    ]
  }

  maybeEscape(dir: number): boolean {
    this.view.focus()
    const targetPos = this.getPos() + (dir < 0 ? 0 : this.node.nodeSize)
    const selection = Selection.near(
      this.view.state.doc.resolve(targetPos),
      dir
    )
    this.view.dispatch(
      this.view.state.tr.setSelection(selection).scrollIntoView()
    )
    return false
  }

  forwardSelection(): void {
    if (!this.cm?.hasFocus) return

    const state = this.view.state
    const selection = this.asProseMirrorSelection(state.doc)
    if (!selection.eq(state.selection)) {
      this.view.dispatch(state.tr.setSelection(selection))
    }
  }

  asProseMirrorSelection(doc: Node): TextSelection {
    const offset = this.getPos() + 1
    const anchor = offset + (this.cm?.state.selection.main.anchor ?? 0)
    const head = offset + (this.cm?.state.selection.main.head ?? 0)
    return TextSelection.create(doc, anchor, head)
  }

  // FIX: This doesn't seem to get called, but doesn't seem to be necessary
  //   setSelection(anchor: number, head: number): void {
  //     console.log('setting selection:', anchor, head)
  //
  //     this.cm?.focus()
  //     this.updating = true
  //     // this.cm?.setSelection(
  //     //   this.cm?.posFromIndex(anchor),
  //     //   this.cm?.posFromIndex(head)
  //     // )
  //     this.updating = false
  //   }

  computeChange(
    oldVal: string,
    newVal: string
  ): null | { from: number; to: number; text: string } {
    if (oldVal === newVal) return null

    let start = 0
    let oldEnd = oldVal.length
    let newEnd = newVal.length

    while (
      start < oldEnd &&
      oldVal.charCodeAt(start) === newVal.charCodeAt(start)
    ) {
      ++start
    }

    while (
      oldEnd > start &&
      newEnd > start &&
      oldVal.charCodeAt(oldEnd - 1) === newVal.charCodeAt(newEnd - 1)
    ) {
      oldEnd--
      newEnd--
    }

    return { from: start, to: oldEnd, text: newVal.slice(start, newEnd) }
  }

  valueChanged(): void {
    if (!this.updating) {
      const change = this.computeChange(
        this.node.textContent,
        this.cm?.state.doc.toString() ?? ''
      )

      if (change && change.text !== '') {
        const start = this.getPos() + 1

        const changeTransaction = this.view.state.tr.replaceWith(
          start + change.from,
          start + change.to,
          articleSchema.nodes.CodeBlock.schema.text(change.text)
        )

        this.view.dispatch(changeTransaction)
      }

      // TODO: Check whether it's necessary
      // this.forwardSelection()
    }

    this.incomingChanges = false
  }

  update(node: Node): boolean {
    if (!isEqual(this.node.type, node.type)) {
      return false
    }

    this.node = node

    const change = node.textContent !== this.cm?.state.doc.toString()

    const changeRange = this.computeChange(
      this.node.textContent,
      this.cm?.state.doc.toString() ?? ''
    )

    if (change && changeRange) {
      this.updating = true

      this.dom
        .setStateFromString(node.textContent)
        .then(() => {
          // Set cursor to changed location within the code editor
          const changeTransaction = this.cm?.state.update({
            selection: {
              anchor: changeRange.to,
            },
          })

          if (changeTransaction) {
            this.cm?.dispatch(changeTransaction)
          }
        })
        .catch((err) => {
          console.log('could not set editor state\n', err)
        })

      this.updating = false
    }

    return true
  }

  selectNode(): void {
    this.cm?.focus()
  }

  deselectNode(): void {
    if (!this.view.hasFocus()) {
      this.view.focus()
    }
  }

  stopEvent(e: Event): boolean {
    return !e.type.startsWith('drag')
  }

  destroy(): void {
    this.cm?.destroy()
    this.dom.remove()
  }
}
