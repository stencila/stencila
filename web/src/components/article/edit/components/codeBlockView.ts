import { Text } from '@codemirror/text'
import { EditorView as CMEditorView } from '@codemirror/view'
import { setAssetPath } from '@stencila/components/dist/components'
import { StencilaEditor } from '@stencila/components/dist/components/stencila-editor'
import { Keymap } from '@stencila/components/dist/types/components/editor/editor'
import { exitCode } from 'prosemirror-commands'
import { redo, undo } from 'prosemirror-history'
import { Node } from 'prosemirror-model'
import { Selection, TextSelection } from 'prosemirror-state'
import { EditorView, NodeView } from 'prosemirror-view'

setAssetPath(
  'https://unpkg.com/@stencila/components@latest/dist/stencila-components/'
)

export class CodeBlockView implements NodeView {
  node: Node
  view: EditorView
  getPos: () => number
  incomingChanges: boolean
  updating: boolean
  ref: StencilaEditor
  cm: CMEditorView | null = null
  doc: CMEditorView | null = null
  dom: HTMLElement | null

  constructor(node: Node, view: EditorView, getPos: boolean | (() => number)) {
    console.log('CODEBLOCK nodeView', node, view)

    // Store for later
    this.node = node
    this.view = view
    if (typeof getPos === 'boolean') {
      this.getPos = () => 0
    } else {
      this.getPos = getPos
    }
    this.incomingChanges = false

    // Create a CodeMirror instance
    // this.cm = new CodeMirror(null, {
    //   value: this.node.textContent,
    //   lineNumbers: true,
    //   extraKeys: this.codeMirrorKeymap(),
    // })

    this.ref = new StencilaEditor()
    this.dom = this.ref

    this.ref.contents = node.textContent
    this.ref.keymap = this.codeMirrorKeymap()
    this.ref.activeLanguage = node.attrs.programmingLanguage

    this.ref.addEventListener('setLanguage', (e) => {
      // TODO: Update node language
      console.log('activeLanguageChanged: ', e)
    })

    this.ref.getRef().then((ref) => {
      this.cm = ref
    })

    // The editor's outer node is our DOM representation
    // this.dom = this.ref.parentElement
    // console.log(this.ref?.parentElement)

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

  forwardSelection() {
    // if (!this.cm?.hasFocus()) return
    let state = this.view.state
    let selection = this.asProseMirrorSelection(state.doc)
    if (!selection.eq(state.selection))
      this.view.dispatch(state.tr.setSelection(selection))
  }

  asProseMirrorSelection(doc: Node) {
    let offset = this.getPos() + 1
    // let anchor = this.cm?.indexFromPos(this.cm?.getCursor('anchor')) + offset
    // let head = this.cm?.indexFromPos(this.cm?.getCursor('head')) + offset
    let anchor = offset
    let head = offset
    return TextSelection.create(doc, anchor, head)
  }

  setSelection(anchor: number, head: number) {
    this.cm?.focus()
    this.updating = true
    // this.cm?.setSelection(
    //   this.cm?.posFromIndex(anchor),
    //   this.cm?.posFromIndex(head)
    // )
    this.updating = false
  }

  computeChange(oldVal: string, newVal: string) {
    if (oldVal === newVal) return null
    let start = 0,
      oldEnd = oldVal.length,
      newEnd = newVal.length
    while (
      start < oldEnd &&
      oldVal.charCodeAt(start) == newVal.charCodeAt(start)
    )
      ++start
    while (
      oldEnd > start &&
      newEnd > start &&
      oldVal.charCodeAt(oldEnd - 1) == newVal.charCodeAt(newEnd - 1)
    ) {
      oldEnd--
      newEnd--
    }
    return { from: start, to: oldEnd, text: newVal.slice(start, newEnd) }
  }

  valueChanged() {
    // TODO: Fix undefined
    let change = this.computeChange(
      this.node.textContent,
      this.cm?.state.doc.toString() ?? ''
    )
    if (change) {
      let start = this.getPos() + 1
      let tr = this.view.state.tr.replaceWith(
        start + change.from,
        start + change.to,
        // TODO: Fix
        // change.text ? codeBlockSchema.text(change.text) : null
        // baseSchema()
        // @ts-ignore
        null
      )
      this.view.dispatch(tr)
    }
  }

  codeMirrorKeymap(): Keymap[] {
    let view = this.view
    return [
      { key: 'ArrowUp', run: () => this.maybeEscape('line', -1) },
      { key: 'ArrowLeft', run: () => this.maybeEscape('char', -1) },
      { key: 'ArrowDown', run: () => this.maybeEscape('line', 1) },
      { key: 'ArrowRight', run: () => this.maybeEscape('char', 1) },
      {
        key: 'Ctrl-Enter',
        run: () => {
          if (exitCode(view.state, view.dispatch)) {
            view.focus()
          }
          return true
        },
      },
      {
        key: 'Mod-Z',
        run: () => {
          undo(view.state, view.dispatch)
          return true
        },
      },
      {
        key: 'Shift-Mod-Z',
        run: () => {
          redo(view.state, view.dispatch)
          return true
        },
      },
      {
        key: 'Mod-Y',
        run: () => {
          redo(view.state, view.dispatch)
          return true
        },
      },
    ]
  }

  maybeEscape(unit: 'line' | 'char', dir: number): boolean {
    this.view.focus()
    let targetPos = this.getPos() + (dir < 0 ? 0 : this.node.nodeSize)
    let selection = Selection.near(this.view.state.doc.resolve(targetPos), dir)
    this.view.dispatch(
      this.view.state.tr.setSelection(selection).scrollIntoView()
    )
    return false
  }

  update(node: Node) {
    if (node.type !== this.node.type) return false
    this.node = node
    // let change = this.computeChange(this.cm?.getValue(), node.textContent)
    const change = this.cm?.state.doc.eq(Text.of([node.textContent]))
    if (change) {
      this.updating = true

      // this.cm?.replaceRange(
      //   change.text,
      //   this.cm?.posFromIndex(change.from),
      //   this.cm?.posFromIndex(change.to)
      // )

      this.ref.setStateFromString(node.textContent)
      this.updating = false
    }
    return true
  }

  selectNode() {
    this.cm?.focus()
  }

  stopEvent() {
    return true
  }
}
