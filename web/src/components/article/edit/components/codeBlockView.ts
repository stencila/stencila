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

export class CodeBlockView implements NodeView {
  cm: CMEditorView | null = null
  dom: HTMLStencilaEditorElement
  getPos: () => number
  ignoreMutation: NodeView['ignoreMutation']
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

    // this.ignoreMutation = (m) => {
    //   return m.type !== 'attributes'
    // }

    this.dom = document.createElement('stencila-editor')

    this.dom.contents = node.textContent
    this.dom.keymap = this.codeMirrorKeymap()
    this.dom.activeLanguage =
      typeof node.attrs.programmingLanguage === 'string'
        ? node.attrs.programmingLanguage
        : ''
    this.dom.contentChangeHandler = () => this.valueChanged()

    this.dom.addEventListener('setLanguage', (e) => {
      const evt = e as CustomEvent<{ name: string }>

      console.log('active Language Changed: ', evt.detail, this.getPos())

      const tr = this.view.state.tr.setNodeMarkup(this.getPos(), undefined, {
        programmingLanguage: evt.detail.name,
      })

      this.view.dispatch(tr)
    })

    this.dom
      .getRef()
      .then((ref) => {
        this.cm = ref
      })
      .catch((err) => {
        console.log(err)
      })

    // The editor's outer node is our DOM representation
    // this.dom = this.dom.parentElement
    // console.log(this.dom?.parentElement)

    // CodeMirror needs to be in the DOM to properly initialize, so
    // schedule it to update itself
    // setTimeout(() => this.cm?.refresh(), 20)

    // This flag is used to avoid an update loop between the outer and
    // inner editor
    this.updating = false

    // Track whether changes are have been made but not yet propagated
    // this.cm?.on('beforeChange', () => (this.incomingChanges = true))

    // Propagate updates from the code editor to ProseMirror
    // this.cm?.on('cursorActivity', () => {
    //   if (!this.updating && !this.incomingChanges) this.forwardSelection()
    // })

    // this.cm?.on('changes', () => {
    //   if (!this.updating) {
    //     this.valueChanged()
    //     this.forwardSelection()
    //   }
    //   this.incomingChanges = false
    // })

    // this.cm?.on('focus', () => this.forwardSelection())
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
        run: () => {
          undo(view.state, dispatch)
          return true
        },
        preventDefault: true,
      },
      {
        key: 'Shift-Mod-z',
        run: () => {
          redo(view.state, dispatch)
          return true
        },
      },
      {
        key: 'Mod-Y',
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
    // if (!this.cm?.hasFocus()) return
    const state = this.view.state
    const selection = this.asProseMirrorSelection(state.doc)
    if (!selection.eq(state.selection))
      this.view.dispatch(state.tr.setSelection(selection))
  }

  asProseMirrorSelection(doc: Node): TextSelection {
    const offset = this.getPos() + 1
    // const anchor = this.cm?.indexFromPos(this.cm?.getCursor('anchor')) + offset
    // const head = this.cm?.indexFromPos(this.cm?.getCursor('head')) + offset
    const anchor = offset
    const head = offset
    return TextSelection.create(doc, anchor, head)
  }

  setSelection(anchor: number, head: number): void {
    this.cm?.focus()
    this.updating = true
    // this.cm?.setSelection(
    //   this.cm?.posFromIndex(anchor),
    //   this.cm?.posFromIndex(head)
    // )
    this.updating = false
  }

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
  }

  update(node: Node): boolean {
    // console.log('updating', isEqual(this.node.type, node.type), node)
    if (!isEqual(this.node.type, node.type)) {
      return false
    }

    // console.log('update: ', node, decorations)
    // console.log('this.type: ', this.node.type)
    // console.log('node.type: ', node.type)
    // console.log('type: ', node.type !== this.node.type)
    // console.log('type: ', isEqual(this.node.type, node.type))
    // console.log('END -------------')

    this.node = node

    // let change = this.computeChange(this.cm?.getValue(), node.textContent)
    // console.log('updating: PRE: change:', node.textContent)
    // console.log('updating: PRE: change:', this.cm?.state.doc)
    // console.log('updating: PRE: change:', this.cm?.state.doc.toString())
    // console.log('updating: PRE: change:', Text.of([node.textContent]))
    // const change = this.cm?.state.doc.eq(Text.of([node.textContent]))
    const change = node.textContent !== this.cm?.state.doc.toString()
    // console.log('updating: change:', change)

    if (change) {
      const changeRange = this.computeChange(
        this.node.textContent,
        this.cm?.state.doc.toString() ?? ''
      )

      this.updating = true

      // TODO: Apply atomic text change
      // this.cm?.replaceRange(
      //   change.text,
      //   this.cm?.posFromIndex(change.from),
      //   this.cm?.posFromIndex(change.to)
      // )

      // const range = EditorSelection.cursor(changeRange.to)

      this.dom.setStateFromString(node.textContent).catch((err) => {
        console.log('could not update editor state\n', err)
      })

      //       const tr = this.cm?.state.changeByRange((range) => ({
      //         range: range,
      //         changes: [
      //           {
      //             from: changeRange.from,
      //             to: changeRange.to,
      //             insert: changeRange.text,
      //           },
      //         ],
      //       }))
      //
      //       if (tr) {
      //         this.cm?.dispatch(tr)
      //       }

      this.updating = false
    }

    return true
  }

  selectNode(): void {
    this.cm?.focus()
  }

  deselectNode(): void {
    this.dom.blur()
  }

  stopEvent(e: Event): boolean {
    return !e.type.startsWith('drag')
  }

  destroy(): void {
    this.cm?.destroy()
    // TODO: Check if it's necessary to remove this.dom
    // this.dom.remove()
  }
}
